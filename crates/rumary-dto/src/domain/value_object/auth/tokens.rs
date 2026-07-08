use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TokenId(pub Uuid);
#[derive(Clone, Debug)]
pub struct TokenHash(String);

impl TokenHash {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }
    pub fn expose(&self) -> &str { &self.0 }
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