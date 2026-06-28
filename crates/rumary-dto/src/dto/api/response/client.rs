use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherClientDto { //JSON
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: String,
    pub loader_version: String,
}