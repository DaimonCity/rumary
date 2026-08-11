use std::fmt::{Display, Formatter};

const MAX_LEN_CONTEXT_KEY: usize = 64;
const MAX_LEN_CONTEXT_VALUE: usize = 128;

/// Ключ условия контекста — `tenant`, `env`, `region`.
/// Нормализуется в нижний регистр, чтобы `Tenant` и `tenant` были одним ключом.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(String);

impl ContextKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContextKey {
    type Error = ContextKeyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LEN_CONTEXT_KEY {
            return Err(Self::Error::InvalidLength);
        }

        if value
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.')
        {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl TryFrom<&str> for ContextKey {
    type Error = ContextKeyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ContextKey> for String {
    fn from(value: ContextKey) -> Self {
        value.0
    }
}

impl Display for ContextKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Значение условия контекста — `acme`, `prod`. Регистр сохраняется: значения
/// приходят извне (id тенанта и т.п.) и сравниваются как есть.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextValue(String);

impl ContextValue {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContextValue {
    type Error = ContextValueError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LEN_CONTEXT_VALUE {
            return Err(Self::Error::InvalidLength);
        }

        if value.chars().any(char::is_control) {
            return Err(Self::Error::InvalidSymbols);
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for ContextValue {
    type Error = ContextValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ContextValue> for String {
    fn from(value: ContextValue) -> Self {
        value.0
    }
}

impl Display for ContextValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKeyError {
    InvalidLength,
    InvalidSymbols,
}

impl Display for ContextKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("context key must be 1..=64 characters long"),
            Self::InvalidSymbols => {
                f.write_str("context key allows only ascii alphanumerics, '_' and '-'")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextValueError {
    InvalidLength,
    InvalidSymbols,
}

impl Display for ContextValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => f.write_str("context value must be 1..=128 characters long"),
            Self::InvalidSymbols => f.write_str("context value must not contain control characters"),
        }
    }
}
