use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

/// Момент истечения ноды/членства в группе.
///
/// Отдельный тип от `api::value_object::auth::ExpirationTime`: тот запрещает
/// прошедшее время (нельзя выдать уже истёкший токен), а здесь прошедшее время
/// — нормальное состояние: истёкшая нода лежит в БД до чистки и просто
/// перестаёт действовать. Хранит `DateTime<Utc>`, а не unix-секунды, чтобы не
/// терять точность на конверсиях с `TIMESTAMPTZ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeExpiry(DateTime<Utc>);

impl NodeExpiry {
    pub fn new(at: DateTime<Utc>) -> Self {
        Self(at)
    }

    pub fn get(self) -> DateTime<Utc> {
        self.0
    }

    pub fn is_expired_at(self, now: DateTime<Utc>) -> bool {
        self.0 <= now
    }

    pub fn is_expired(self) -> bool {
        self.is_expired_at(Utc::now())
    }
}

impl From<DateTime<Utc>> for NodeExpiry {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}

impl From<NodeExpiry> for DateTime<Utc> {
    fn from(value: NodeExpiry) -> Self {
        value.0
    }
}

impl Display for NodeExpiry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
