use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(Uuid);

impl From<Uuid> for InstanceId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<InstanceId> for Uuid {
    fn from(value: InstanceId) -> Self {
        value.0
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
