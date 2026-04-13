use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub auth_source: AuthSource,
    pub banned: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthSource {
    Local,
    External { provider: String, subject: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: Uuid,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedFile {
    pub path: String,
    pub checksum: Option<String>,
    pub size: Option<u64>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathRuleSet {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub minecraft_version: String,
    pub authlib_injector_url: Option<String>,
    pub files: Vec<ManagedFile>,
    pub rules: PathRuleSet,
    pub launch_arguments: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub client_id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub mods: Vec<ManagedFile>,
    pub rules: PathRuleSet,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherBuild {
    pub id: Uuid,
    pub version: Version,
    pub channel: String,
    pub download_url: String,
    pub checksum: Option<String>,
    pub changelog: Option<String>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub profile_id: Option<Uuid>,
    pub platform: String,
    pub launcher_version: Option<Version>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationPlan {
    pub installation_id: Uuid,
    pub client: Client,
    pub profile: Option<Profile>,
    pub files: Vec<ManagedFile>,
    pub launch_arguments: Vec<String>,
    pub auth_endpoint: String,
    pub skin_service_url: Option<String>,
    pub update_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherUpdate {
    pub update_available: bool,
    pub latest: Option<LauncherBuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibConfig {
    pub auth_server_url: String,
    pub session_server_url: String,
    pub services_server_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkinServiceConfig {
    pub base_url: Option<String>,
}
