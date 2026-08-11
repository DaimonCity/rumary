use crate::domain::api::value_object::user::HashError;
use bcrypt::{DEFAULT_COST, hash, verify};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TokenId(Uuid);

impl TokenId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<TokenId> for Uuid {
    fn from(id: TokenId) -> Self {
        id.0
    }
}

impl From<Uuid> for TokenId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Clone, Debug)]
pub struct TokenHash(String);

impl TokenHash {
    pub fn new(token: String) -> Result<Self, HashError> {
        let hash = hash(token, DEFAULT_COST).map_err(HashError::HashingFailed)?;
        Ok(Self(hash))
    }

    pub fn verify(&self, token: &str) -> Result<bool, HashError> {
        verify(&self.0, token).map_err(HashError::VerifyingFailed)
    }

    pub fn from_stored(value: String) -> Self {
        Self(value)
    }
}

impl From<TokenHash> for String {
    fn from(value: TokenHash) -> Self {
        value.0
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Display for TokenHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}
