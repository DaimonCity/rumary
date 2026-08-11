use crate::domain::perms::value_object::group::{GroupName, GroupWeight};
use crate::domain::perms::value_object::resource::{ResourceId, ResourceType};
use crate::domain::perms::value_object::user::UserId;
use std::fmt::{Display, Formatter};

/// Ссылка на конкретную запись ресурса — пара (тип, id). Раньше эти два
/// значения передавались отдельными `&str`-аргументами и их было легко
/// перепутать местами; теперь это один тип, который нельзя собрать неправильно.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceRef {
    resource_type: ResourceType,
    resource_id: ResourceId,
}

impl ResourceRef {
    pub fn new(resource_type: ResourceType, resource_id: ResourceId) -> Self {
        Self {
            resource_type,
            resource_id,
        }
    }

    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl Display for ResourceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource_type, self.resource_id)
    }
}

/// Получатель доступа к ресурсу:
/// - `Role` — конкретная роль (по имени, включая унаследованные группы);
/// - `User` — конкретный пользователь точечно;
/// - `MinWeight` — "эта роль и выше" по весу, без перечисления ролей поимённо
///   (например, "writer и все роли с weight >= writer.weight").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessGrant {
    Group(GroupName),
    User(UserId),
    MinWeight(GroupWeight),
}

impl AccessGrant {
    pub fn holder_type(&self) -> HolderType {
        match self {
            Self::Group(_) => HolderType::Group,
            Self::User(_) => HolderType::User,
            Self::MinWeight(_) => HolderType::MinWeight,
        }
    }

    /// Значение колонки `resource_access.holder_id` для этого гранта.
    pub fn holder_id(&self) -> String {
        match self {
            Self::Group(name) => name.to_string(),
            Self::User(id) => id.to_string(),
            Self::MinWeight(weight) => weight.to_string(),
        }
    }
}

impl Display for AccessGrant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.holder_type(), self.holder_id())
    }
}

/// Тип держателя доступа — соответствует CHECK-ограничению на
/// `resource_access.holder_type` и `permission_nodes.holder_type`.
/// Enum вместо строкового литерала: опечатка в `'min_weigth'` больше не
/// доедет до БД, чтобы там развалиться на constraint-е.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HolderType {
    User,
    Group,
    MinWeight,
}

impl HolderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Group => "role",
            Self::MinWeight => "min_weight",
        }
    }
}

impl Display for HolderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Держатель ноды прав в `permission_nodes` — только user или group.
/// Отдельный тип от `HolderType`: в `permission_nodes` нет `min_weight`,
/// и CHECK-ограничение там другое.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeHolderType {
    User,
    Group,
}

impl NodeHolderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Group => "group",
        }
    }
}

impl Display for NodeHolderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Право записи в ресурс (`resource_access.can_write`). Именованный тип
/// вместо голого `bool` в позиции последнего аргумента `grant(...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
}

impl AccessMode {
    pub fn can_write(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

impl From<bool> for AccessMode {
    fn from(can_write: bool) -> Self {
        if can_write {
            Self::ReadWrite
        } else {
            Self::ReadOnly
        }
    }
}

impl Display for AccessMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => f.write_str("read-only"),
            Self::ReadWrite => f.write_str("read-write"),
        }
    }
}
