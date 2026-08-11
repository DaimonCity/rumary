use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub struct ConfigurationId(Uuid);

impl From<Uuid> for ConfigurationId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ConfigurationId> for Uuid {
    fn from(value: ConfigurationId) -> Self {
        value.0
    }
}

impl Display for ConfigurationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
