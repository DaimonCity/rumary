use crate::domain::perms::value_object::context::{ContextKeyError, ContextValueError};
use crate::domain::perms::value_object::group::{GroupNameError, GroupWeightError};
use crate::domain::perms::value_object::node::PermissionKeyError;
use crate::domain::perms::value_object::resource::{ResourceIdError, ResourceTypeError};
use std::fmt::{Display, Formatter};

/// Локальный аналог `err_from!` из `domain::api`: тот макрос экспортируется
/// только при включённой фиче `domain_api`, а perms должен собираться
/// независимо от неё.
macro_rules! perms_err_from {
    ($from:ty, $variant:ident) => {
        impl From<$from> for PermsValueObjectError {
            fn from(value: $from) -> Self {
                Self::$variant(value)
            }
        }
    };
}

/// Сборный тип ошибок валидации value object-ов подсистемы прав — по аналогии
/// с `api::value_object::error::ValueObjectError`. Нужен, чтобы хендлеры
/// маппили одну ошибку в 400, а не десять по отдельности.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermsValueObjectError {
    ContextKey(ContextKeyError),
    ContextValue(ContextValueError),
    GroupName(GroupNameError),
    GroupWeight(GroupWeightError),
    PermissionKey(PermissionKeyError),
    ResourceType(ResourceTypeError),
    ResourceId(ResourceIdError),
}

perms_err_from!(ContextKeyError, ContextKey);
perms_err_from!(ContextValueError, ContextValue);
perms_err_from!(GroupNameError, GroupName);
perms_err_from!(GroupWeightError, GroupWeight);
perms_err_from!(PermissionKeyError, PermissionKey);
perms_err_from!(ResourceTypeError, ResourceType);
perms_err_from!(ResourceIdError, ResourceId);

impl Display for PermsValueObjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextKey(err) => err.fmt(f),
            Self::ContextValue(err) => err.fmt(f),
            Self::GroupName(err) => err.fmt(f),
            Self::GroupWeight(err) => err.fmt(f),
            Self::PermissionKey(err) => err.fmt(f),
            Self::ResourceType(err) => err.fmt(f),
            Self::ResourceId(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for PermsValueObjectError {}
