use crate::domain::api::Loader::*;
use crate::domain::version::Version;

#[derive(Default, Clone, Debug)]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric(Version),
    Forge(Version),
    NeoForge(Version),
}

impl From<Loader> for String {
    fn from(value: Loader) -> Self {
        match value {
            Vanilla => "vanilla".into(),
            Fabric(_) => "fabric".into(),
            Forge(_) => "forge".into(),
            NeoForge(_) => "neoforge".into(),
        }
    }
}

impl Loader {
    pub fn from_string(
        loader: String,
        loader_version: Option<Version>,
    ) -> Result<Self, LoaderError> {
        if let Some(loader_version) = loader_version {
            match loader.as_str() {
                "vanilla" => Ok(Vanilla),
                "fabric" => Ok(Fabric(loader_version)),
                "forge" => Ok(Forge(loader_version)),
                "neoforge" => Ok(NeoForge(loader_version)),
                _ => Err(LoaderError::MissingLoader),
            }
        } else {
            Err(LoaderError::MissingVersion)
        }
    }
    pub fn get_version(&self) -> Option<Version> {
        match self {
            Vanilla => None,
            Fabric(v) | Forge(v) | NeoForge(v) => Some(v.clone()),
        }
    }
}

#[derive(Debug)]
pub enum LoaderError {
    MissingLoader,
    MissingVersion,
}
