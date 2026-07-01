use crate::domain::launcher::state::OsType;
use std::error::Error;

impl TryFrom<&str> for OsType {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "windows" => Ok(OsType::Windows),
            "linux" => Ok(OsType::Linux),
            "macos" => Ok(OsType::MacOs),
            _ => Err(Box::<dyn Error + Send + Sync>::from("Unknown OS type.")),
        }
    }
}