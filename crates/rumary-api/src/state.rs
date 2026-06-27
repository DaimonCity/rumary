use std::sync::Arc;
use crate::auth::AuthService;
use crate::totp::TotpService;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub totp: Arc<TotpService>,
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