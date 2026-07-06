use crate::domain::api::Loader::*;

#[derive(Default, Clone, Debug)]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric(String),
    Forge(String),
    NeoForge(String)
}

impl Loader {
    pub fn from_strings(loader: String, loader_version: Option<String>) -> Self {
        match loader.to_lowercase().as_str() {
            "forge" => Forge(loader_version.unwrap_or("missing".into())),
            "fabric" => Fabric(loader_version.unwrap_or("missing".into())),
            "neoforge" => NeoForge(loader_version.unwrap_or("missing".into())),
            _ => Vanilla,
        }
    }
}