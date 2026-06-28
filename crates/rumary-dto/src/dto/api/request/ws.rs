use serde::{Deserialize, Serialize};
use crate::dto::api::request::auth::AccessLevelRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct WsTicketClaimsRequest {
    pub sub: String,
    pub level: AccessLevelRequest,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}
