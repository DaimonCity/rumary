use crate::domain::api::AccessLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WsTicketClaims {
    pub sub: String,
    pub level: AccessLevel,
    pub ver: i32,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WsTicketV2Claims {
    pub sub: String,
    pub ver: i32,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}
