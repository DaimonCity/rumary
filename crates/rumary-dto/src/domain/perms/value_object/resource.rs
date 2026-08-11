use crate::domain::perms::value_object::node::{PermissionKey, PermissionKeyError};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

const MAX_LEN_RESOURCE_TYPE: usize = 64;
const MAX_LEN_RESOURCE_ID: usize = 128;
const BYPASS_ACL_SEGMENT: &str = "bypass_acl";

/// Тип ресурса в общей ACL-таблице — `configuration`, `instance`, `profile`.
///
/// Это значение попадает и в `resource_access.resource_type`, и в ключи прав
/// (`{resource_type}.bypass_acl`), поэтому набор допустимых символов совпадает
/// с сегментом `PermissionKey`: точки и `*` запрещены.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceType(String);

impl ResourceType {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Ключ RBAC-обхода ACL для этого типа ресурса: `configuration.bypass_acl`.
    /// Права на него достаточно, чтобы не проверять ACL вообще.
    pub fn bypass_acl_key(&self) -> PermissionKey {
        Self::build_key(&self.0, BYPASS_ACL_SEGMENT)
    }

    /// Ключ права на действие с этим типом ресурса: `configuration.get`.
    pub fn action_key(&self, action: &str) -> Result<PermissionKey, PermissionKeyError> {
        PermissionKey::try_from(format!("{}.{action}", self.0))
    }

    /// Оба сегмента уже провалидированы (тип ресурса — при создании,
    /// константа — на этапе компиляции), поэтому ключ не может быть невалидным.
    fn build_key(resource_type: &str, segment: &str) -> PermissionKey {
        PermissionKey::try_from(format!("{resource_type}.{segment}"))
            .expect("resource type and segment are pre-validated")
    }
}

impl TryFrom<String> for ResourceType {
    type Error = ResourceTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LEN_RESOURCE_TYPE {
            return Err(Self::Error::InvalidLength);
        }

        if value
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.')
        {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl TryFrom<&str> for ResourceType {
    type Error = ResourceTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ResourceType> for String {
    fn from(value: ResourceType) -> Self {
        value.0
    }
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Идентификатор конкретной записи ресурса. В БД хранится текстом, потому что
/// одна ACL-таблица обслуживает разные таблицы ресурсов — но у нас почти
/// всегда это Uuid, поэтому есть `From<Uuid>` без валидации.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Uuid> for ResourceId {
    fn from(value: Uuid) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<String> for ResourceId {
    type Error = ResourceIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LEN_RESOURCE_ID {
            return Err(Self::Error::InvalidLength);
        }

        if value.chars().any(char::is_control) {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for ResourceId {
    type Error = ResourceIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ResourceId> for String {
    fn from(value: ResourceId) -> Self {
        value.0
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceTypeError {
    InvalidLength,
    InvalidSymbols,
}

impl Display for ResourceTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("resource type must be 1..=64 characters long"),
            Self::InvalidSymbols => {
                f.write_str("resource type allows only ascii alphanumerics, '_' and '-'")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceIdError {
    InvalidLength,
    InvalidSymbols,
}

impl Display for ResourceIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("resource id must be 1..=128 characters long"),
            Self::InvalidSymbols => f.write_str("resource id must not contain control characters"),
        }
    }
}
