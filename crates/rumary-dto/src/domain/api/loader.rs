use crate::domain::api::Loader::*;
use crate::domain::api::value_object::version::Version;

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
        match loader.as_str() {
            "vanilla" => Ok(Vanilla),
            "fabric" => loader_version.map(Fabric).ok_or(LoaderError::MissingVersion),
            "forge" => loader_version.map(Forge).ok_or(LoaderError::MissingVersion),
            "neoforge" => loader_version.map(NeoForge).ok_or(LoaderError::MissingVersion),
            _ => Err(LoaderError::MissingLoader),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_does_not_require_loader_version() {
        assert!(matches!(
            Loader::from_string("vanilla".to_owned(), None),
            Ok(Loader::Vanilla)
        ));
    }

    #[test]
    fn modded_loader_requires_version() {
        assert!(matches!(
            Loader::from_string("fabric".to_owned(), None),
            Err(LoaderError::MissingVersion)
        ));
    }
}
