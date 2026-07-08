use crate::error::AppError;
use crate::service::file::FileService;
use crate::service::totp::TotpService;
use crate::services::{AuthProvider, UserProfileProvider};
use std::sync::Arc;
use crate::service::settings::SettingsService;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthProvider<Error = AppError>>,
    pub user_profile: Arc<dyn UserProfileProvider<Error = AppError>>,
    pub totp: Arc<TotpService>,
    pub file: Arc<FileService>,
    pub settings: Arc<SettingsService>,
    pub secure_cookies: bool,
}