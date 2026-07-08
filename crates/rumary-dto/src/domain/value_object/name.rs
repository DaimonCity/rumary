use std::path::{Path, PathBuf};

const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const RESERVED: [&str; 6] = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
pub struct DirectoryName(PathBuf);

impl AsRef<Path> for DirectoryName {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
impl TryFrom<String> for DirectoryName {
    type Error = DirectoryNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // 1. Проверка длины (обычно до 255 символов)
        if value.is_empty() || value.len() > 255 {
            return Err(Self::Error::InvalidLength);
        }

        // 2. Проверка на запрещенные символы
        if value
            .chars()
            .any(|c| FORBIDDEN_CHARS.contains(&c) || c.is_control())
        {
            return Err(Self::Error::InvalidCharacters);
        }

        // 3. Проверка на зарезервированные имена (Windows-compatibility)
        if RESERVED.contains(&value.to_uppercase().as_str()) {
            return Err(Self::Error::ReservedName);
        }

        Ok(Self(PathBuf::from(value)))
    }
}
impl From<DirectoryName> for String {
    fn from(name: DirectoryName) -> String {
        name.0.as_path().to_string_lossy().into()
    }
}
#[derive(Debug)]
pub enum DirectoryNameError {
    InvalidLength,
    InvalidCharacters,
    ReservedName,
}

pub struct DisplayName(String);

impl TryFrom<String> for DisplayName {
    type Error = DisplayNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > 255 {
            return Err(Self::Error::InvalidLength);
        }

        Ok(Self(value))
    }
}
impl From<DisplayName> for String {
    fn from(name: DisplayName) -> String {
        name.0
    }
}
#[derive(Debug)]
pub enum DisplayNameError {
    InvalidLength,
    InvalidCharacters,
    ReservedName,
}

pub struct Description(String);
impl TryFrom<String> for Description {
    type Error = DescriptionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > 500 {
            return Err(Self::Error::InvalidLength)
        }

        Ok(Self(value))
    }
}

impl From<Description> for String {
    fn from(value: Description) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub enum DescriptionError {
    InvalidLength,
}