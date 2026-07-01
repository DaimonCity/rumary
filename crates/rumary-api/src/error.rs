use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("token error: {0}")]
    Token(String),
    #[error("io error: {0}")]
    Io(std::io::Error),
    #[error("migration error: {0}")]
    Migration(MigrateError),
    #[error("config error: {0}")]
    Configuration(String),
    #[error("uuid error: {0}")]
    Uuid(uuid::Error),
    #[error("fmt error: {0}")]
    Fmt(std::fmt::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Database(_)
            | Self::Internal(_)
            | Self::Configuration(_)
            | Self::Crypto(_)
            | Self::Uuid(_)
            | Self::Fmt(_)
            | Self::Token(_)
            | Self::Io(_)
            | Self::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<std::fmt::Error> for AppError {
    fn from(err: std::fmt::Error) -> Self {
        AppError::Fmt(err)
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Uuid(err)
    }
}

impl From<MigrateError> for AppError {
    fn from(value: MigrateError) -> Self {
        Self::Migration(value)
    }
}



pub type AppResult<T> = Result<T, AppError>;
