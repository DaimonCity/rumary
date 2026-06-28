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
    pub fn from_strings(loader: String, loader_version: String) -> Self {
        match loader.to_lowercase().as_str() {
            "forge" => Forge(loader_version),
            "fabric" => Fabric(loader_version),
            "neoforge" => NeoForge(loader_version),
            _ => Vanilla,
        }
    }
}