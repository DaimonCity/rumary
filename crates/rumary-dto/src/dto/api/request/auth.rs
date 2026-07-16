use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleTypeRequest {
    User = 0,
    VipUser = 1,
    Worker = 2,
    Owner = 3,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsV2Request {
    pub sub: String,
    pub level: Vec<usize>,
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
    pub user_id: Uuid,
    pub totp_code: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMeRequest {
    pub password: String,
}
