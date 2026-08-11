use std::fmt::{Display, Formatter};

const MIN_LEN_GROUP_NAME: usize = 2;
const MAX_LEN_GROUP_NAME: usize = 64;

/// Имя группы (роли) — `user`, `manager`, `admin`, `writer`.
///
/// В БД имя группы — это внешний ключ по значению (`user_groups.group_name`,
/// `group_inheritance.parent_name`, `permission_nodes.holder_id`), поэтому
/// нормализуем регистр здесь: иначе `Admin` и `admin` стали бы двумя разными
/// ролями с разными правами.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupName(String);

impl GroupName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GroupName {
    type Error = GroupNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Self::Error::Missing);
        }

        if MIN_LEN_GROUP_NAME > value.len() || value.len() > MAX_LEN_GROUP_NAME {
            return Err(Self::Error::InvalidLength);
        }

        if value
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl TryFrom<&str> for GroupName {
    type Error = GroupNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<GroupName> for String {
    fn from(value: GroupName) -> Self {
        value.0
    }
}

impl Display for GroupName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Вес группы: приоритет при конфликте прав между группами одного
/// пользователя и одновременно "ранг" для проверок `actor_outranks_target`.
///
/// Не может быть отрицательным — иначе сравнение с весом пользователя без
/// групп (0) начинает вести себя неочевидно.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupWeight(i32);

impl GroupWeight {
    /// Вес пользователя, не входящего ни в одну группу.
    pub const NONE: Self = Self(0);

    pub fn new(value: i32) -> Result<Self, GroupWeightError> {
        if value < 0 {
            return Err(GroupWeightError::Negative);
        }
        Ok(Self(value))
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for GroupWeight {
    type Error = GroupWeightError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GroupWeight> for i32 {
    fn from(value: GroupWeight) -> Self {
        value.0
    }
}

impl Display for GroupWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupNameError {
    InvalidLength,
    InvalidSymbols,
    Missing,
}

impl Display for GroupNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("group name must be 2..=64 characters long"),
            Self::InvalidSymbols => {
                f.write_str("group name allows only ascii alphanumerics, '_' and '-'")
            }
            Self::Missing => f.write_str("group name is missing"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupWeightError {
    Negative,
}

impl Display for GroupWeightError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negative => f.write_str("group weight must not be negative"),
        }
    }
}
