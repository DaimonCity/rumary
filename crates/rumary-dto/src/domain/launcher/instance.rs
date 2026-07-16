use crate::domain::launcher::Configuration;
use uuid::Uuid;

pub struct Instance {
    pub id: Uuid,
    pub profiles: Vec<Configuration>,
}
