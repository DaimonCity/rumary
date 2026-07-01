use crate::error::{AppError, AppResult};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
    pub jwt_secret: String,
    pub totp_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub ws_ticket_ttl_seconds: i64,
    pub secure_cookies: bool,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let host = read_env("API_HOST", "0.0.0.0");
        let port = parse_env("API_PORT", "3000")?;
        let secure_cookies = parse_env("COOKIE_SECURE", "false")?;
        let access_token_ttl_minutes = parse_env("ACCESS_TOKEN_TTL_MINUTES", "15")?;
        let refresh_token_ttl_days = parse_env("REFRESH_TOKEN_TTL_DAYS", "30")?;
        let ws_ticket_ttl_seconds = parse_env("WS_TICKET_TTL_SECONDS", "60")?;

        let database = DatabaseConfig {
            host: read_env("DB_HOST", "0.0.0.0"),
            port: parse_env("DB_PORT", "5432")?,
            user: read_env("DB_USER", "postgres"),
            password: read_env("DB_PASSWORD", "postgres"),
            db_name: read_env("DB_NAME", "postgres"),
        };

        let jwt_secret = required_env("JWT_SECRET")?;
        let totp_secret = required_env("TOTP_SECRET")?;

        if totp_secret.len() != 32 {
            return Err(AppError::Configuration(
                "TOTP_SECRET must be exactly 32 bytes long".to_string(),
            ));
        }

        Ok(Self {
            host,
            port,
            database,
            jwt_secret,
            totp_secret,
            access_token_ttl_minutes,
            refresh_token_ttl_days,
            ws_ticket_ttl_seconds,
            secure_cookies,
        })
    }

    pub fn totp_secret_key(&self) -> [u8; 32] {
        self.totp_secret
            .as_bytes()
            .try_into()
            .expect("validated in Config::from_env")
    }
}

fn required_env(key: &str) -> AppResult<String> {
    env::var(key).map_err(|_| AppError::Configuration(format!("missing required env var `{key}`")))
}

fn read_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_env<T>(key: &str, default: &str) -> AppResult<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = read_env(key, default);
    raw.parse::<T>()
        .map_err(|err| AppError::Configuration(format!("invalid value for `{key}`: {err}")))
}
