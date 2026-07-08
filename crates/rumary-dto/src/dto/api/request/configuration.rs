use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationsRequest {
    pub instance_id: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewConfigurationRequest {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub instance_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigurationRequest {
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub instance_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigurationRequest {
    pub configuration_id: Uuid,
}