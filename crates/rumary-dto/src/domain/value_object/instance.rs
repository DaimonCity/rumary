use uuid::Uuid;

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