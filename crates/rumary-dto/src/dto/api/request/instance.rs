use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewInstanceRequest {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: String,
    pub loader_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInstanceRequest {
    pub uuid: Uuid,
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetInstanceRequest {
    pub instance_uuid: Uuid,
}