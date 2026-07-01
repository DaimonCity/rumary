use serde::{Deserialize, Serialize};
use crate::domain::api::AccessLevel;

#[derive(Debug, Serialize, Deserialize)]
pub struct WsTicketClaims {
    pub sub: String,
    pub level: AccessLevel,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}
