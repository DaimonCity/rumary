use crate::error::{AppError, AppResult};
use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode, header},
};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub struct FileService {
    namespaces: HashMap<String, PathBuf>,
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileService {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
        }
    }

    // Метод для регистрации папок при старте приложения
    pub fn register_namespace(mut self, name: &str, path: &str) -> Self {
        self.namespaces
            .insert(name.to_string(), PathBuf::from(path));
        self
    }

    pub async fn stream_file(
        &self,
        namespace: &str,
        filename: &str,
        headers: &HeaderMap,
    ) -> AppResult<Response<Body>> {
        // 1. Валидация имени файла от Directory Traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(AppError::Validation("Forbidden path".into()));
        }

        // 2. Ищем, зарегистрирована ли такая папка (пространство имен)
        let base_path = self.namespaces.get(namespace).ok_or_else(|| {
            AppError::Internal(format!("FileService: namespace '{}' not found", namespace))
        })?;
        let file_path = base_path.join(filename);

        // 3. Проверяем метаданные файла
        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_err(|_| AppError::NotFound("FileService: metadata is lost".to_string()))?;

        // 4. Логика HTTP-кэширования (ETag) для файлов 10-100 МБ
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

        // 5. Открываем и стримим файл чанками
        let file = File::open(file_path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::NotFound("FileService: file not found".to_string())
            }
            _ => AppError::Internal(e.to_string()),
        })?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        // 6. Формируем ответ
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
