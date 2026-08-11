use crate::domain::api::BanScope;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserBanRequest {
    pub scope: BanScope,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason_code: String,
    pub staff_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeUserBanRequest {
    pub reason: String,
}
