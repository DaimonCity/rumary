use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherConfig {
    pub version: u8,
    pub client_path: String,
    pub api_url: String,
    pub username: String,
    pub access_token: String,
    pub uuid: String,
}

impl LauncherConfig {
    fn _new(
        version: u8,
        client_path: String,
        api_url: String,
        username: String,
        access_token: String,
        uuid: String,
    ) -> Self {
        Self {
            version,
            client_path,
            api_url,
            username,
            access_token,
            uuid,
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            version: 1,
            client_path: default_client_path(),
            api_url: "http://127.0.0.1:3000".to_string(),
            username: "Player".to_string(),
            access_token: "token".to_string(),
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

fn default_client_path() -> String {
    if cfg!(windows) {
        match std::env::var("APPDATA") {
            Ok(path) => format!("{path}/.rumary"),
            Err(_) => ".rumary".to_string(),
        }
    } else {
        match std::env::var("HOME") {
            Ok(path) => format!("{path}/.rumary"),
            Err(_) => ".rumary".to_string(),
        }
    }
}
