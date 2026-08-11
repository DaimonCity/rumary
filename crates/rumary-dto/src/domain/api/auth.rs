use crate::domain::api::value_object::auth::expiration_time::ExpirationTime;
use crate::domain::api::value_object::auth::tokens::{TokenHash, TokenId};
use crate::dto::api::response::{SessionTokensResponse, TotpRequiredResponse};
use serde::{Deserialize, Serialize};

pub enum LoginOutcome {
    Tokens(SessionTokensResponse),
    TotpRequired(TotpRequiredResponse),
}

#[derive(Clone, Debug)]
pub struct RefreshSessionUpdate {
    pub token_id: TokenId,
    pub refresh_token_hash: TokenHash,
    pub expires_at: ExpirationTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleType {
    User = 0,
    VipUser = 1,
    Worker = 2,
    Owner = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessLevel {
    pub role_type: RoleType,
    pub level: u16, // Конкретный уровень внутри промежутка
}
