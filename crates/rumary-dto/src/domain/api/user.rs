use crate::domain::api::AccessLevel;
use crate::domain::api::value_object::auth::tokens::{TokenHash, TokenId};
use crate::domain::api::value_object::user::{Login, Nickname, PasswordHash, UserId};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub login: Login,
    pub nickname: Nickname,
    pub password_hash: PasswordHash,
    pub access_level: AccessLevel,
    pub token_version: i32,
    pub is_public: bool,
}

pub struct UserSession {
    pub id: UserId,
    pub token_id: TokenId,
    pub refresh_token_hash: TokenHash,
    pub expires_at: DateTime<Utc>,
}

pub struct NewUser {
    pub login: Login,
    pub nickname: Nickname,
    pub password_hash: PasswordHash,
}
#[derive(Debug, Clone)]
pub struct TotpUser {
    pub id: UserId,
    pub totp: String,
    pub step: i64,
    pub nonce: String,
    pub confirmed: bool,
}

pub struct NewTotpUser {
    pub user_id: Uuid,
    pub encrypted_secret: String,
    pub nonce: String,
}
