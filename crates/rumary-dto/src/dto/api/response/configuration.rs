use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationsResponse {
    pub configurations: Vec<GetConfigurationResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigurationResponse {
    pub uuid: Uuid,
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub instance_uuid: String,

    pub hard_dirs: Vec<String>,
    pub soft_dirs: Vec<String>,
    pub files: HashMap<String, File>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    sha1: String,

    #[serde(rename = "ruma_serde::time::ms_since_unix_epoch")]
    _type: FileType,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    Required,
    Optional,
}