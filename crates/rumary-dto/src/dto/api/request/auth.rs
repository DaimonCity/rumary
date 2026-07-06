use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleTypeRequest {
    User = 0,
    VipUser = 1,
    Builder = 2,
    Writer = 3,
    Admin = 4,
    Owner = 5
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AccessLevelRequest {
    pub role_type: RoleTypeRequest,
    pub level: u16,
}
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub nickname: String,
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsRequest {
    pub sub: String,
    pub level: AccessLevelRequest,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpLoginRequest {
    pub user_uuid: Uuid,
    pub totp_code: String,
}
