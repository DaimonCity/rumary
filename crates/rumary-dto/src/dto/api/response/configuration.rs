use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationDto {
    pub display_name: String,
    pub client_uuid: String,
    pub dir_name: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckDirsDto {
    #[serde(flatten)]
    pub dirs: HashMap<String, FilesDto>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilesDto {
    #[serde(flatten)]
    pub files: HashMap<String, FileInfoDto>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfoDto {
    pub sha1: String,
    #[serde(rename = "type")]
    pub _type: CheckTypeDto,
    pub path: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckTypeDto {
    Required,
    Optional,
}