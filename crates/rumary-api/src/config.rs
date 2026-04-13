use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub bind_addr: SocketAddr,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let database_url = match env::var("DATABASE_URL") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => build_database_url_from_parts()?,
        };

        let host = env::var("API_HOST")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let port = env::var("API_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3000);

        Ok(Self {
            database_url,
            bind_addr: SocketAddr::new(host, port),
        })
    }
}

fn build_database_url_from_parts() -> AppResult<String> {
    let host = env_or_default("DB_HOST", "127.0.0.1");
    let port = env_or_default("DB_PORT", "5000");
    let user = env_or_default("DB_USER", "postgres");
    let password = env_or_default("DB_PASSWORD", "postgres");
    let db_name = env_or_default("DB_NAME", "postgres");

    Ok(format!(
        "postgres://{user}:{password}@{host}:{port}/{db_name}"
    ))
}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}
