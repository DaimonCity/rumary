use std::fmt::{Display, Formatter};

const MAX_LEN_PERMISSION_KEY: usize = 255;
const WILDCARD: &str = "*";

/// Ключ ноды прав — `configuration.get`, `api.orders.*`, `*`.
///
/// Точки разделяют сегменты, `*` в сегменте закрывает весь хвост. Валидация
/// на входе гарантирует, что резолверу не придётся разбирать мусорные ключи:
/// пустые сегменты (`api..read`) и `*` внутри сегмента (`api.ord*`) отсекаются
/// здесь, а не молча превращаются в "ничего не совпало".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionKey(String);

impl PermissionKey {
    /// Нода "всё разрешено" — то, что выдаётся owner-у.
    pub fn wildcard() -> Self {
        Self(WILDCARD.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('.')
    }

    /// Содержит ли ключ wildcard-сегмент — такая нода покрывает поддерево,
    /// а не одно конкретное право.
    pub fn is_wildcard(&self) -> bool {
        self.segments().any(|segment| segment == WILDCARD)
    }

    /// Дочерний ключ: `configuration` + `get` -> `configuration.get`.
    /// Используется для сборки ключей вида `{resource_type}.bypass_acl`,
    /// где префикс уже провалидирован.
    pub fn join(&self, segment: &str) -> Result<Self, PermissionKeyError> {
        Self::try_from(format!("{}.{segment}", self.0))
    }
}

impl TryFrom<String> for PermissionKey {
    type Error = PermissionKeyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LEN_PERMISSION_KEY {
            return Err(Self::Error::InvalidLength);
        }

        for segment in value.split('.') {
            if segment.is_empty() {
                return Err(Self::Error::EmptySegment);
            }

            if segment == WILDCARD {
                continue;
            }

            if segment.contains(WILDCARD) {
                return Err(Self::Error::PartialWildcard);
            }

            if segment
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
            {
                return Err(Self::Error::InvalidSymbols);
            }
        }

        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl TryFrom<&str> for PermissionKey {
    type Error = PermissionKeyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<PermissionKey> for String {
    fn from(value: PermissionKey) -> Self {
        value.0
    }
}

impl Display for PermissionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKeyError {
    InvalidLength,
    EmptySegment,
    PartialWildcard,
    InvalidSymbols,
}

impl Display for PermissionKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("permission key must be 1..=255 characters long"),
            Self::EmptySegment => f.write_str("permission key must not contain empty segments"),
            Self::PartialWildcard => {
                f.write_str("wildcard '*' must occupy a whole segment, not a part of it")
            }
            Self::InvalidSymbols => {
                f.write_str("permission key allows only ascii alphanumerics, '_', '-' and '*'")
            }
        }
    }
}

/// Приоритет источника ноды: у прямой ноды пользователя он заведомо выше
/// любого веса группы. Тип отдельный от `GroupWeight`, чтобы вес группы
/// нельзя было по ошибке передать вместо приоритета и наоборот.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePriority(i32);

impl SourcePriority {
    /// Приоритет прямой ноды пользователя — выше любой группы.
    pub const USER: Self = Self(1_000_000);
    /// Нейтральный приоритет для нод без источника (тесты, ручная сборка).
    pub const ZERO: Self = Self(0);

    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl From<i32> for SourcePriority {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<SourcePriority> for i32 {
    fn from(value: SourcePriority) -> Self {
        value.0
    }
}

impl Display for SourcePriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
