pub struct TotpCode(String);

impl TotpCode {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for TotpCode {
    type Error = TotpCodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 6 {
            return Err(Self::Error::LengthMismatch);
        }

        if value.chars().any(|c| !c.is_ascii_digit()) {
            return Err(Self::Error::InvalidChars);
        }

        Ok(TotpCode(value))
    }
}

#[derive(Debug)]
pub enum TotpCodeError {
    LengthMismatch,
    InvalidChars,
}

impl std::fmt::Display for TotpCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
