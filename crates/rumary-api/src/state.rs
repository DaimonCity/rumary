use std::sync::Arc;

use chrono::Utc;
use semver::Version;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{
        Client, InstallationPlan, InstallationRequest, LauncherBuild, LauncherUpdate, ManagedFile,
        PathRuleSet, Profile, User, ValidationIssue, ValidationReport,
    },
    repository::AppRepository,
    services::{AuthProvider, MinecraftProvider, SkinService},
};

pub struct AppState {
    pub repository: Arc<dyn AppRepository>,
    pub auth_provider: Arc<dyn AuthProvider>,
    pub minecraft_provider: Arc<dyn MinecraftProvider>,
    pub skin_service: Arc<dyn SkinService>,
}

impl AppState {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        auth_provider: Arc<dyn AuthProvider>,
        minecraft_provider: Arc<dyn MinecraftProvider>,
        skin_service: Arc<dyn SkinService>,
    ) -> Self {
        Self {
            repository,
            auth_provider,
            minecraft_provider,
            skin_service,
        }
    }

    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        self.repository.list_users().await
    }

    pub async fn get_user(&self, user_id: Uuid) -> AppResult<User> {
        self.repository
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user `{user_id}`")))
    }

    pub async fn set_user_ban(&self, user_id: Uuid, banned: bool) -> AppResult<User> {
        self.repository.update_user_ban(user_id, banned).await
    }

    pub async fn create_client(&self, mut client: Client) -> AppResult<Client> {
        validate_slug(&client.slug)?;
        let report = validate_paths(&client.files, &client.rules);
        if !report.valid {
            return Err(AppError::Validation(format!(
                "client path validation failed: {:?}",
                report.issues
            )));
        }

        client.id = Uuid::new_v4();
        client.created_at = Utc::now();
        self.repository.insert_client(&client).await?;
        Ok(client)
    }

    pub async fn list_clients(&self) -> AppResult<Vec<Client>> {
        self.repository.list_clients().await
    }

    pub async fn get_client(&self, client_id: Uuid) -> AppResult<Client> {
        self.repository
            .find_client_by_id(client_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("client `{client_id}`")))
    }

    pub async fn create_profile(&self, mut profile: Profile) -> AppResult<Profile> {
        validate_slug(&profile.slug)?;
        let client_exists = self
            .repository
            .find_client_by_id(profile.client_id)
            .await?
            .is_some();
        if !client_exists {
            return Err(AppError::NotFound(format!(
                "client `{}` for profile",
                profile.client_id
            )));
        }

        let report = validate_paths(&profile.mods, &profile.rules);
        if !report.valid {
            return Err(AppError::Validation(format!(
                "profile path validation failed: {:?}",
                report.issues
            )));
        }

        profile.id = Uuid::new_v4();
        profile.created_at = Utc::now();
        self.repository.insert_profile(&profile).await?;
        Ok(profile)
    }

    pub async fn get_profile(&self, profile_id: Uuid) -> AppResult<Profile> {
        self.repository
            .find_profile_by_id(profile_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("profile `{profile_id}`")))
    }

    pub async fn validate_client(&self, client_id: Uuid) -> AppResult<ValidationReport> {
        let client = self.get_client(client_id).await?;
        Ok(validate_paths(&client.files, &client.rules))
    }

    pub async fn validate_profile(&self, profile_id: Uuid) -> AppResult<ValidationReport> {
        let profile = self.get_profile(profile_id).await?;
        Ok(validate_paths(&profile.mods, &profile.rules))
    }

    pub async fn publish_launcher_build(
        &self,
        mut build: LauncherBuild,
    ) -> AppResult<LauncherBuild> {
        build.id = Uuid::new_v4();
        build.published_at = Utc::now();
        self.repository.insert_launcher_build(&build).await?;
        Ok(build)
    }

    pub async fn get_latest_launcher_build(
        &self,
        channel: &str,
    ) -> AppResult<Option<LauncherBuild>> {
        self.repository.latest_launcher_build(channel).await
    }

    pub async fn check_launcher_update(
        &self,
        channel: &str,
        current_version: &Version,
    ) -> AppResult<LauncherUpdate> {
        let latest = self.get_latest_launcher_build(channel).await?;
        let update_available = latest
            .as_ref()
            .map(|build| build.version > *current_version)
            .unwrap_or(false);
        Ok(LauncherUpdate {
            update_available,
            latest,
        })
    }

    pub async fn create_installation(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        profile_id: Option<Uuid>,
        platform: String,
        launcher_version: Option<Version>,
    ) -> AppResult<InstallationPlan> {
        let user = self.get_user(user_id).await?;
        if user.banned {
            return Err(AppError::Unauthorized("user is banned".into()));
        }

        let client = self.get_client(client_id).await?;
        let profile = match profile_id {
            Some(id) => {
                let profile = self.get_profile(id).await?;
                if profile.client_id != client.id {
                    return Err(AppError::Validation(
                        "profile does not belong to requested client".into(),
                    ));
                }
                Some(profile)
            }
            None => None,
        };

        let installation_id = Uuid::new_v4();
        let request = InstallationRequest {
            id: installation_id,
            user_id,
            client_id,
            profile_id,
            platform,
            launcher_version,
            created_at: Utc::now(),
        };
        self.repository
            .insert_installation_request(&request)
            .await?;

        let mut files = client.files.clone();
        if let Some(profile) = &profile {
            files.extend(profile.mods.clone());
        }

        let skin_service = self.skin_service.get_config().await?;

        Ok(InstallationPlan {
            installation_id,
            client,
            profile,
            files,
            launch_arguments: vec![
                "--username".into(),
                user.username,
                "--version".into(),
                "rumary-managed".into(),
            ],
            auth_endpoint: "/api/integrations/authlib".into(),
            skin_service_url: skin_service.base_url,
            update_url: "/api/launcher/updates/check".into(),
        })
    }
}

fn validate_slug(slug: &str) -> AppResult<()> {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(AppError::Validation(format!(
            "invalid slug `{slug}`; expected lowercase ascii, digits, `-` or `_`"
        )));
    }
    Ok(())
}

pub fn validate_paths(files: &[ManagedFile], rules: &PathRuleSet) -> ValidationReport {
    use std::collections::HashMap;

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut issues = Vec::new();

    for file in files {
        let normalized = normalize_path(&file.path);
        let count = seen.entry(normalized.clone()).or_insert(0);
        *count += 1;

        let is_blacklisted = matches_rule(&normalized, &rules.blacklist);
        let is_whitelisted = matches_rule(&normalized, &rules.whitelist);

        if *count > 1 && (!is_whitelisted || is_blacklisted) {
            issues.push(ValidationIssue {
                path: normalized,
                reason: if is_blacklisted {
                    "duplicate path is forbidden by blacklist".into()
                } else {
                    "duplicate path outside whitelist".into()
                },
            });
        }
    }

    ValidationReport {
        valid: issues.is_empty(),
        issues,
        checked_at: Utc::now(),
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
        .to_ascii_lowercase()
}

fn matches_rule(path: &str, rules: &[String]) -> bool {
    rules.iter().any(|rule| {
        let normalized_rule = normalize_path(rule);
        path == normalized_rule || path.starts_with(&format!("{normalized_rule}/"))
    })
}

#[cfg(test)]
mod tests {
    use crate::models::{ManagedFile, PathRuleSet};

    use super::validate_paths;

    #[test]
    fn duplicate_path_is_allowed_inside_whitelist() {
        let report = validate_paths(
            &[
                ManagedFile {
                    path: "mods/shared/file.jar".into(),
                    checksum: None,
                    size: None,
                    download_url: None,
                },
                ManagedFile {
                    path: "mods/shared/file.jar".into(),
                    checksum: None,
                    size: None,
                    download_url: None,
                },
            ],
            &PathRuleSet {
                whitelist: vec!["mods/shared".into()],
                blacklist: vec![],
            },
        );

        assert!(report.valid);
    }

    #[test]
    fn blacklist_overrides_whitelist_for_duplicate_paths() {
        let report = validate_paths(
            &[
                ManagedFile {
                    path: "mods/shared/critical.jar".into(),
                    checksum: None,
                    size: None,
                    download_url: None,
                },
                ManagedFile {
                    path: "mods/shared/critical.jar".into(),
                    checksum: None,
                    size: None,
                    download_url: None,
                },
            ],
            &PathRuleSet {
                whitelist: vec!["mods/shared".into()],
                blacklist: vec!["mods/shared/critical.jar".into()],
            },
        );

        assert!(!report.valid);
        assert_eq!(report.issues.len(), 1);
    }
}
