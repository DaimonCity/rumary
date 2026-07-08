use serde::{ Serialize};
use uuid::Uuid;
use crate::domain::value_object::auth::tokens::TokenId;

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimsResponse {
    pub sub: String,
    pub level: AccessLevelResponse,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize)]
pub struct TotpRequiredResponse {
    pub user_id: Uuid,
}
#[derive(Debug, Clone)]
pub struct SessionTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_token_id: TokenId,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleTypeResponse {
    User = 0,
    VipUser = 1,
    Worker = 2,
    Owner = 3
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct AccessLevelResponse {
    pub role_type: RoleTypeResponse,
    pub level: u16,
}