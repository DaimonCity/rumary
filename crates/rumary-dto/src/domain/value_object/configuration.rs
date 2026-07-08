use uuid::Uuid;

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
