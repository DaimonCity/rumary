use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherClientDto { //JSON
    pub id: Uuid,
    pub name: String,
    pub icon: String,
    pub version: String,
    pub url: String,
    pub loader: String,
    pub loader_version: String,
    pub profiles: String,
}