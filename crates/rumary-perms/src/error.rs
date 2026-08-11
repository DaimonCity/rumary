use rumary_dto::domain::perms::value_object::error::PermsValueObjectError;
use rumary_dto::domain::perms::value_object::group::{GroupNameError, GroupWeightError};
use rumary_dto::domain::perms::value_object::node::{PermissionKey, PermissionKeyError};

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    /// Право не выдано (или выдан явный запрет). Хранит сам ключ, а не строку:
    /// вызывающий код может понять, какого именно права не хватило, без
    /// парсинга текста ошибки.
    #[error("permission denied: {0}")]
    Denied(PermissionKey),

    /// Действие отклонено не из-за отсутствия ноды, а из-за ранга: actor не
    /// выше target по весу группы.
    #[error("insufficient rank: {0}")]
    InsufficientRank(&'static str),

    #[error("failed to load permissions: {0}")]
    StoreError(sqlx::Error),

    /// Невалидное значение на входе (ключ права, имя группы, контекст).
    #[error("invalid permission value: {0}")]
    InvalidValue(PermsValueObjectError),
}

impl From<PermsValueObjectError> for PermissionError {
    fn from(value: PermsValueObjectError) -> Self {
        Self::InvalidValue(value)
    }
}

impl From<GroupNameError> for PermissionError {
    fn from(value: GroupNameError) -> Self {
        Self::InvalidValue(PermsValueObjectError::GroupName(value))
    }
}
impl From<PermissionKeyError> for PermissionError {
    fn from(value: PermissionKeyError) -> Self {
        Self::InvalidValue(PermsValueObjectError::PermissionKey(value))
    }
}

impl From<GroupWeightError> for PermissionError {
    fn from(value: GroupWeightError) -> Self {
        Self::InvalidValue(PermsValueObjectError::GroupWeight(value))
    }
}

impl From<sqlx::Error> for PermissionError {
    fn from(value: sqlx::Error) -> Self {
        Self::StoreError(value)
    }
}

pub type PermissionResult<T> = Result<T, PermissionError>;
