use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Downloads {
    pub client: DownloadInfo,
    pub client_mappings: Option<DownloadInfo>,
    pub server: Option<DownloadInfo>,
    pub server_mappings: Option<DownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadInfo {
    pub path: Option<String>,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}