use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rumary_dto::domain::api::LoaderError;
use rumary_dto::domain::api::share_target::ShareTargetError;
use rumary_dto::domain::api::value_object::auth::errors::ExpirationTimeError;
use rumary_dto::domain::api::value_object::error::ValueObjectError;
use rumary_dto::domain::api::value_object::name::{
    DescriptionError, DirectoryNameError, DisplayNameError,
};
use rumary_dto::domain::api::value_object::url::IconUrlError;
use rumary_dto::domain::api::value_object::user::{HashError, LoginError, NicknameError};
use rumary_dto::domain::api::value_object::version::VersionError;
use rumary_dto::domain::perms::value_object::error::PermsValueObjectError;
use rumary_dto::domain::perms::value_object::group::{GroupNameError, GroupWeightError};
use rumary_dto::domain::perms::value_object::resource::ResourceTypeError;
use rumary_dto::err_from;
use rumary_perms::PermissionError;
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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("banned: {0}")]
    Banned(String),
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
    #[error("json error: {0}")]
    JsonError(serde_json::error::Error),
    #[error("resource type error: {0}")]
    ResourceTypeError(ResourceTypeError),
    #[error("share target error")]
    ShareTargetError(ShareTargetError),
    #[error("invalid perms value")]
    InvalidPermsValue(PermsValueObjectError),
    #[error("invalid group name")]
    InvalidGroupName(GroupNameError),
    #[error("invalid group weight")]
    InvalidGroupWeight(GroupWeightError),
    #[error("init totp error: {0}")]
    TotpError(totp_rs::TotpError)
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
            | Self::ShareTargetError(_)
            | Self::InvalidGroupName(_)
            | Self::InvalidGroupWeight(_)
            | Self::InvalidPermsValue(_)
            | Self::InvalidDirectoryName(_)
            | Self::InvalidDisplayName(_)
            | Self::InvalidVersion(_)
            | Self::InvalidLogin(_)
            | Self::InvalidNickname(_)
            | Self::InvalidLoader(_)
            | Self::InvalidDescription(_)
            | Self::InvalidIconUrl(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Banned(_) => StatusCode::FORBIDDEN,
            Self::TokenExpired(_) => StatusCode::UNAUTHORIZED,
            Self::Database(_)
            | Self::Internal(_)
            | Self::Configuration(_)
            | Self::Crypto(_)
            | Self::Uuid(_)
            | Self::JsonError(_)
            | Self::Fmt(_)
            | Self::InvalidHash(_)
            | Self::Token(_)
            | Self::ResourceTypeError(_)
            | Self::Io(_)
            | Self::TotpError(_)
            | Self::Url(_)
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
err_from!(std::io::Error, AppError, Io);
err_from!(serde_json::error::Error, AppError, JsonError);
err_from!(ResourceTypeError, AppError, ResourceTypeError);
err_from!(ShareTargetError, AppError, ShareTargetError);
err_from!(GroupWeightError, AppError, InvalidGroupWeight);
err_from!(GroupNameError, AppError, InvalidGroupName);
err_from!(totp_rs::TotpError, AppError, TotpError);

impl From<PermissionError> for AppError {
    fn from(err: PermissionError) -> Self {
        match err {
            PermissionError::Denied(e) => Self::Forbidden(e.to_string()),
            PermissionError::InsufficientRank(e) => Self::Forbidden(e.to_string()),
            PermissionError::StoreError(e) => Self::Database(e),
            PermissionError::InvalidValue(e) => Self::InvalidPermsValue(e),
        }
    }
}

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
