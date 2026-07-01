use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::dto::api::response::{SessionTokensResponse, TotpRequiredResponse};


pub enum LoginOutcome {
    Tokens(SessionTokensResponse),
    TotpRequired(TotpRequiredResponse),
}

#[derive(Clone, Debug)]
pub struct RefreshSessionUpdate {
    pub token_id: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleType {
    User = 0,
    VipUser = 1,
    Builder = 2,
    Writer = 3,
    Admin = 4,
    Owner = 5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessLevel {
    pub role_type: RoleType,
    pub level: u8, // Конкретный уровень внутри промежутка
}

#[derive(Debug, Deserialize)]
pub struct DeleteMeRequest {
    pub password: String,
}
