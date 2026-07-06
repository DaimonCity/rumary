use uuid::Uuid;
use crate::domain::launcher::Configuration;

pub struct Instance {
    pub id: Uuid,
    pub profiles: Vec<Configuration>,
}