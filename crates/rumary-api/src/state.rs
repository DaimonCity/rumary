use crate::error::AppError;
use crate::service::file::FileService;
use crate::service::totp::TotpService;
use crate::services::{AuthProvider, UserProfileProvider};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthProvider<Error = AppError>>,
    pub user_profile: Arc<dyn UserProfileProvider<Error = AppError>>,
    pub totp: Arc<TotpService>,
    pub file: Arc<FileService>,
    pub secure_cookies: bool,
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
