use crate::error::{AppError, AppResult};
use crate::repo::repository::{ConfigurationRepository, InstanceRepository, SettingsRepository};
use axum::{
    body::Body,
    http::{HeaderMap, Response},
};
use http::{StatusCode, header};
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use rumary_dto::domain::configuration::ConfigurationId;

pub struct FileService {
    configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
    settings_repo: Arc<dyn SettingsRepository<Error = AppError>>,
}

impl FileService {
    pub fn new(
        configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
        instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
        settings_repo: Arc<dyn SettingsRepository<Error = AppError>>,
    ) -> Self {
        Self {
            configuration_repo,
            instance_repo,
            settings_repo,
        }
    }

    pub async fn stream_file(
        &self,
        configuration_uuid: ConfigurationId,
        filepath: &Path,
        headers: &HeaderMap,
        access_level: u16,
    ) -> AppResult<Response<Body>> {
        // 1. Ищем, зарегистрирована ли такая папка (пространство имен)
        // путь до папки instances, где все instance
        // root_path -> /home/rumary/instances/
        let root_path = self.settings_repo.get_instances_dir_path().await?;

        // /home/rumary/instances/LeKRAFT 1.0
        // /home/rumary/instances/LeKRAFT 2.0
        // /home/rumary/instances/LeKRAFT Test
        let configuration = self
            .configuration_repo
            .get_config(configuration_uuid, access_level)
            .await?;
        let instance = self
            .instance_repo
            .get_instance(configuration.instance_id, access_level)
            .await?;
        let instance_path = root_path.join(instance.dir_name);

        // /home/rumary/instances/LeKRAFT 1.0/Potato
        // /home/rumary/instances/LeKRAFT 1.0/Medium
        // /home/rumary/instances/LeKRAFT 1.0/High
        let configuration_path = instance_path.join(configuration.dir_name);
        let file_path = configuration_path.join(filepath);

        // 2. Проверяем метаданные файла
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_err(|_| AppError::NotFound("FileService: metadata is lost".to_string()))?;

        // 3. Логика HTTP-кэширования (ETag) для файлов 10-100 МБ
        // Безопасно получаем ETag, если ОС поддерживает время модификации файла
        let etag = match metadata.modified() {
            Ok(modified) => {
                let duration = modified
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                Some(format!("W/\"{}-{}\"", duration.as_secs(), metadata.len()))
            }
            Err(_) => None, // Если ОС не поддерживает modified_time, отдаем файл без ETag
        };

        // Проверяем ETag, если он успешно сгенерирован
        if let Some(ref etag_str) = etag
            && let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
            && if_none_match == etag_str.as_str()
        {
            return Ok(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())?);
        }

        // 4. Открываем и стримим файл чанками
        let file = File::open(file_path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::NotFound("FileService: file not found".to_string())
            }
            _ => AppError::Internal(e.to_string()),
        })?;

        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        // 5. Формируем ответ
        let mut response_builder = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, metadata.len())
            .header(header::CACHE_CONTROL, "public, max-age=86400"); // Кэш на 1 день

        // Добавляем ETag в заголовки только если он есть
        if let Some(etag_str) = etag {
            response_builder = response_builder.header(header::ETAG, etag_str);
        }

        let response = response_builder.body(body)?;

        Ok(response)
    }
}
