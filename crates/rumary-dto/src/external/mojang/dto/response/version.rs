use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::external::mojang::dto::response::argument::Arguments;
use crate::external::mojang::dto::response::asset::AssetIndex;
use crate::external::mojang::dto::response::download::Downloads;
use crate::external::mojang::dto::response::library::Library;
use crate::external::mojang::dto::response::logging::Logging;
use crate::external::mojang::dto::response::other::JavaVersion;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

impl Version {
    pub async fn get_version(
        version_manifest: &VersionManifest,
        id: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let versions = &version_manifest.versions;
        let version = versions
            .iter()
            .find(|v| v.id == id)
            .ok_or("version not found")?
            .clone();
        Ok(version)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionJson {
    pub arguments: Option<Arguments>,

    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,

    pub assets: String,

    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,

    pub downloads: Downloads,

    pub id: String,

    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,

    pub libraries: Vec<Library>,

    pub logging: Option<Logging>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i32,

    #[serde(rename = "releaseTime")]
    pub release_time: String,

    pub time: String,

    #[serde(rename = "type")]
    pub version_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}