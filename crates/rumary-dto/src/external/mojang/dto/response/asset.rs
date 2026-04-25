use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AssetJson {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,

    #[serde(rename = "totalSize")]
    pub total_size: u64,

    pub url: String,
}