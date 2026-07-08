use bcrypt::{DEFAULT_COST, hash, verify};
use std::fmt::Formatter;
use std::ops::Deref;
use uuid::Uuid;

const MIN_LEN_LOGIN: usize = 3;
const MAX_LEN_LOGIN: usize = 20;

const MIN_LEN_NICKNAME: usize = 3;
const MAX_LEN_NICKNAME: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl Deref for UserId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}
#[derive(Clone, Debug)]
pub struct Login(String);
#[derive(Clone, Debug)]
pub struct Nickname(String);
#[derive(Clone)]
pub struct PasswordHash(String);

// Удобный способ получения сырого значения
impl Login {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Login {
    type Error = LoginError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Self::Error::Missing);
        }

        if MIN_LEN_LOGIN > value.len() && value.len() > MAX_LEN_LOGIN {
            return Err(Self::Error::InvalidLength);
        }
        if value
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
        {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value))
    }
}

impl From<Login> for String {
    fn from(value: Login) -> Self {
        value.0
    }
}

impl Nickname {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Nickname {
    type Error = NicknameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Self::Error::Missing);
        }

        if MIN_LEN_NICKNAME > value.len() || value.len() > MAX_LEN_NICKNAME {
            return Err(Self::Error::InvalidLength);
        }

        if value
            .as_bytes()
            .iter()
            .any(|&b| !b.is_ascii_alphanumeric() && b != b'_')
        {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value))
    }
}
impl From<Nickname> for String {
    fn from(value: Nickname) -> Self {
        value.0
    }
}

impl PasswordHash {
    pub fn new(password: String) -> Result<Self, PasswordHashError> {
        let hash = hash(password, DEFAULT_COST).map_err(PasswordHashError::HashingFailed)?;
        Ok(Self(hash))
    }

    pub fn verify(&self, password: &str) -> Result<bool, PasswordHashError> {
        verify(&self.0, password).map_err(PasswordHashError::VerifyingFailed)
    }
}

impl std::fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("PasswordHash(***)")
    }
}

impl Deref for PasswordHash {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum PasswordHashError {
    VerifyingFailed(bcrypt::BcryptError),
    HashingFailed(bcrypt::BcryptError),
}

#[derive(Debug)]
pub enum NicknameError {
    InvalidLength,
    InvalidSymbols,
    Missing,
}

#[derive(Debug)]
pub enum LoginError {
    InvalidLength,
    InvalidSymbols,
    Missing,
}
