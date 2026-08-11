use crate::error::{AppError, AppResult};
use crate::repo::repository::{ConfigurationRepository, InstanceRepository, SettingsRepository};
use crate::services::FileResolver;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{HeaderMap, Response},
};
use http::{StatusCode, header};
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::impl_new;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub struct FileService {
    resolver: Arc<dyn FileResolver>,
}

impl FileService {
    pub fn new(resolver: Arc<dyn FileResolver>) -> Self {
        Self { resolver }
    }

    pub async fn stream_file(
        &self,
        config_id: ConfigurationId,
        filepath: &Path,
        headers: &HeaderMap,
        // access_level: u16,
    ) -> AppResult<Response<Body>> {
        let handle = self
            .resolver
            .resolve_file(config_id, filepath)
            .await?;

        let (file, len, modified) = match handle {
            FileHandle::LocalV1(path) => {
                let meta = tokio::fs::metadata(&path)
                    .await
                    .map_err(|_| AppError::NotFound("File not found".into()))?;
                let file = File::open(path)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                (file, meta.len(), meta.modified().ok())
            }
        };

        // 2. Логика ETag (теперь используем `modified` и `len`)
        let etag = modified.and_then(|m| {
            m.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .ok()
                .map(|d| format!("W/\"{}-{}\"", d.as_secs(), len))
        });

        // 3. Проверка If-None-Match
        if let Some(etag_str) = &etag
            && let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
            && if_none_match == etag_str.as_str()
        {
            return Ok(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())?);
        }

        // 4. Формирование ответа с заголовками
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, len)
            .header(header::CACHE_CONTROL, "public, max-age=86400");

        if let Some(etag_str) = etag {
            builder = builder.header(header::ETAG, etag_str);
        }

        Ok(builder.body(Body::from_stream(ReaderStream::new(file)))?)
    }
}

pub enum FileHandle {
    LocalV1(PathBuf),
    // В будущем: S3Object { bucket: String, key: String }
}

pub struct LocalFileResolver {
    config_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
    settings_repo: Arc<dyn SettingsRepository<Error = AppError>>,
}

impl_new!(LocalFileResolver,
    config_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
    settings_repo: Arc<dyn SettingsRepository<Error = AppError>>);

#[async_trait]
impl FileResolver for LocalFileResolver {
    async fn resolve_file(
        &self,
        config_id: ConfigurationId,
        requested_path: &Path,
        // access_level: u16,
    ) -> AppResult<FileHandle> {
        let config = self.config_repo.get_config(config_id).await?;
        let root_path = self.settings_repo.get_instances_dir_path().await?;

        let instance = self.instance_repo.get_instance(config.instance_id).await?;
        let path = root_path
            .join(instance.dir_name)
            .join(&config.dir_name)
            .join(requested_path);

        // Тут же можно сделать проверку на Path Traversal (безопасность)
        Ok(FileHandle::LocalV1(path))
    }
}
