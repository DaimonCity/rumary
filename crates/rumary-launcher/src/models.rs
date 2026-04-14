use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherClient {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub id: Uuid,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaunchCommand {
    pub main_class: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub classpath: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionManifest {
    pub latest: Value,
    pub versions: Value,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
    pub client: DownloadInfo,
    pub client_mappings: Option<DownloadInfo>,
    pub server: Option<DownloadInfo>,
    pub server_mappings: Option<DownloadInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Complex {
        rules: Option<Vec<Rule>>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,

    #[serde(rename = "totalSize")]
    pub total_size: u64,

    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: DownloadInfo,
    #[serde(rename = "type")]
    pub log_type: String,
}
