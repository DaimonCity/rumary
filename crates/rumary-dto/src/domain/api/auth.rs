use crate::domain::api::value_object::auth::expiration_time::ExpirationTime;
use crate::domain::api::value_object::auth::tokens::{TokenHash, TokenId};
use crate::dto::api::response::{SessionTokensResponse, TotpRequiredResponse};

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
