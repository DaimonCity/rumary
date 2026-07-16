use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstancesResponse {
    pub instances: Vec<GetInstanceResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInstanceResponse {
    pub id: Uuid,
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: String,
    pub loader_version: Option<String>,
}
