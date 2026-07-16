use crate::service::right::Rights;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rumary_dto::domain::api::{LoaderError, RoleError};
use rumary_dto::domain::auth::errors::ExpirationTimeError;
use rumary_dto::domain::error::ValueObjectError;
use rumary_dto::domain::name::{DescriptionError, DirectoryNameError, DisplayNameError};
use rumary_dto::domain::url::IconUrlError;
use rumary_dto::domain::user::{HashError, LoginError, NicknameError};
use rumary_dto::domain::version::VersionError;
use rumary_dto::err_from;
use serde::Serialize;
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("database error: {0}")]
    Database(sqlx::Error),
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
    #[error("url error: {0}")]
    Url(http::uri::InvalidUri),
    #[error("http error: {0}")]
    Http(http::Error),
    #[error("expiration error")]
    TokenExpired(ExpirationTimeError),
    #[error("invalid directory name")]
    InvalidDirectoryName(DirectoryNameError),
    #[error("invalid display name")]
    InvalidDisplayName(DisplayNameError),
    #[error("invalid icon url")]
    InvalidIconUrl(IconUrlError),
    #[error("invalid version")]
    InvalidVersion(VersionError),
    #[error("invalid description")]
    InvalidDescription(DescriptionError),
    #[error("invalid loader")]
    InvalidLoader(LoaderError),
    #[error("invalid login")]
    InvalidLogin(LoginError),
    #[error("invalid nickname")]
    InvalidNickname(NicknameError),
    #[error("invalid hash")]
    InvalidHash(HashError),
    #[error("role error")]
    RoleError(RoleError),
    #[error("json error: {0}")]
    JsonError(serde_json::error::Error),
    #[error("send rights error: {0}")]
    SendRightsError(SendError<Rights>),
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
            Self::Validation(_)
            | Self::InvalidDirectoryName(_)
            | Self::InvalidDisplayName(_)
            | Self::InvalidVersion(_)
            | Self::InvalidLogin(_)
            | Self::InvalidNickname(_)
            | Self::InvalidLoader(_)
            | Self::InvalidDescription(_)
            | Self::InvalidIconUrl(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::UNAUTHORIZED,
            Self::TokenExpired(_) => StatusCode::UNAUTHORIZED,
            Self::Database(_)
            | Self::Internal(_)
            | Self::Configuration(_)
            | Self::RoleError(_)
            | Self::Crypto(_)
            | Self::Uuid(_)
            | Self::JsonError(_)
            | Self::Fmt(_)
            | Self::InvalidHash(_)
            | Self::Token(_)
            | Self::Io(_)
            | Self::Url(_)
            | Self::SendRightsError(_)
            | Self::Http(_)
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

err_from!(sqlx::Error, AppError, Database);
err_from!(std::fmt::Error, AppError, Fmt);
err_from!(uuid::Error, AppError, Uuid);
err_from!(MigrateError, AppError, Migration);
err_from!(http::Error, AppError, Http);
err_from!(ExpirationTimeError, AppError, TokenExpired);
err_from!(NicknameError, AppError, InvalidNickname);
err_from!(LoginError, AppError, InvalidLogin);
err_from!(HashError, AppError, InvalidHash);
err_from!(DisplayNameError, AppError, InvalidDisplayName);
err_from!(DirectoryNameError, AppError, InvalidDirectoryName);
err_from!(VersionError, AppError, InvalidVersion);
err_from!(DescriptionError, AppError, InvalidDescription);
err_from!(LoaderError, AppError, InvalidLoader);
err_from!(IconUrlError, AppError, InvalidIconUrl);
err_from!(RoleError, AppError, RoleError);
err_from!(std::io::Error, AppError, Io);
err_from!(serde_json::error::Error, AppError, JsonError);
err_from!(SendError<Rights>, AppError, SendRightsError);

impl From<ValueObjectError> for AppError {
    fn from(value: ValueObjectError) -> Self {
        match value {
            ValueObjectError::DirectoryName(e) => Self::InvalidDirectoryName(e),
            ValueObjectError::DisplayName(e) => Self::InvalidDisplayName(e),
            ValueObjectError::Description(e) => Self::InvalidDescription(e),
            ValueObjectError::IconUrl(e) => Self::InvalidIconUrl(e),
            ValueObjectError::Nickname(e) => Self::InvalidNickname(e),
            ValueObjectError::Login(e) => Self::InvalidLogin(e),
            ValueObjectError::PasswordHash(e) => Self::InvalidHash(e),
            ValueObjectError::Version(e) => Self::InvalidVersion(e),
            ValueObjectError::LoaderError(e) => Self::InvalidLoader(e),
        }
    }
}
pub type AppResult<T> = Result<T, AppError>;
