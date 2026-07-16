use crate::external::mojang::dto::response::download::DownloadInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingClient {
    pub argument: String,
    pub file: DownloadInfo,
    #[serde(rename = "type")]
    pub log_type: String,
}
