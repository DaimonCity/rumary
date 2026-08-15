use bcrypt::{DEFAULT_COST, hash, verify};
use std::char::TryFromCharError;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use uuid::Uuid;

const MIN_LEN_LOGIN: usize = 3;
const MAX_LEN_LOGIN: usize = 20;
const MIN_LEN_PASSWORD: usize = 8;
const MAX_LEN_PASSWORD: usize = 36;

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

impl Display for UserId {
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

        if MIN_LEN_LOGIN > value.len() || value.len() > MAX_LEN_LOGIN {
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
    pub fn verify(&self, password: &str) -> Result<bool, PasswordHashError> {
        verify(password, &self.0).map_err(PasswordHashError::VerifyingFailed)
    }

    pub fn from_stored(value: String) -> Self {
        Self(value)
    }
}

impl From<PasswordHash> for String {
    fn from(value: PasswordHash) -> Self {
        value.0
    }
}
impl TryFrom<String> for PasswordHash {
    type Error = PasswordHashError;

    fn try_from(password: String) -> Result<Self, Self::Error> {
        if password.is_empty() {
            return Err(Self::Error::Missing);
        }

        if MIN_LEN_PASSWORD > password.len() || password.len() > MAX_LEN_PASSWORD {
            return Err(Self::Error::InvalidLength);
        }

        let mut at_least_numeric = false;
        let mut at_least_upper_case = false;
        let mut at_least_lower_case = false;
        let mut at_least_special_sym = false;

        for c in password.chars() {
            if c.is_ascii_lowercase() {
                at_least_lower_case = true;
            } else if c.is_ascii_uppercase() {
                at_least_upper_case = true;
            } else if c.is_ascii_digit() {
                at_least_numeric = true;
            } else if c.is_ascii_punctuation() {
                // Проверяем именно на знаки пунктуации/спецсимволы
                at_least_special_sym = true;
            } else {
                return Err(Self::Error::InvalidSymbols);
            }
        }

        if !(at_least_numeric & at_least_upper_case & at_least_lower_case & at_least_special_sym) {
            return Err(Self::Error::InvalidPassword {
                upper_case: at_least_upper_case,
                lower_case: at_least_lower_case,
                digit: at_least_numeric,
                sym: at_least_special_sym,
            });
        }

        let hash = hash(password, DEFAULT_COST).map_err(PasswordHashError::HashingFailed)?;
        Ok(Self(hash))
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
    Missing,
    InvalidLength,
    InvalidSymbols,
    InvalidPassword {
        upper_case: bool,
        lower_case: bool,
        digit: bool,
        sym: bool,
    },
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for PasswordHashError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordHashError::VerifyingFailed(e) => {
                write!(f, "failed to verify password: {}", e)
            }
            PasswordHashError::HashingFailed(e) => {
                write!(f, "failed to get password hash: {}", e)
            }
            PasswordHashError::Missing => {
                write!(f, "missing password")
            }
            PasswordHashError::InvalidLength => {
                write!(f, "invalid password")
            }
            PasswordHashError::InvalidSymbols => {
                write!(f, "invalid password")
            }
            PasswordHashError::InvalidPassword {
                upper_case,
                lower_case,
                digit,
                sym,
            } => {
                write!(
                    f,
                    "invalid password: upper_case is {}, lower_case is {}, digit is {}, sym is {}",
                    upper_case, lower_case, digit, sym
                )
            }
            PasswordHashError::Internal(_) => {
                write!(f, "internal error")
            }
        }
    }
}

impl From<TryFromCharError> for PasswordHashError {
    fn from(e: TryFromCharError) -> Self {
        Self::Internal(Box::new(e))
    }
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
