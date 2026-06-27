use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::api::AccessLevel;

#[derive(Debug, Clone)]
pub struct User {
    pub uuid: Uuid,
    pub login: String,
    pub nickname: String,
    pub password_hash: String,
    pub access_level: AccessLevel,
    pub ban: Ban,
}

pub struct UserSession {
    pub uuid: Uuid,
    pub token_uuid: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
}

pub struct NewUser {
    pub login: String,
    pub nickname: String,
    pub password_hash: String,
}
#[derive(Debug, Clone)]
pub struct TotpUser {
    pub uuid: Uuid,
    pub totp: String,
    pub nonce: String,
    pub confirmed: bool,
}

pub struct NewTotpUser {
    pub uuid: Uuid,
    pub encrypted_secret: String,
    pub nonce: String,
}

impl User {
    pub fn is_banned(&self) -> bool {
        self.ban.collect().contains(&true)
    }
}

#[derive(Debug, Clone)]
pub struct Ban {
    pub ip: bool,
    pub hwid: bool,
}

impl Ban {
    pub fn collect(&self) -> Vec<bool> {
        vec![self.ip, self.hwid]
    }
}