use crate::error::AppError;
use crate::service::configurations::ConfigurationService;
use crate::service::file::FileService;
use crate::service::instances::InstanceService;
use crate::service::roles::RoleService;
use crate::service::settings::SettingsService;
use crate::service::totp::TotpService;
use crate::services::{AuthProvider, UserProfileProvider};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthProvider<Error = AppError>>,
    pub user_profile: Arc<dyn UserProfileProvider<Error = AppError>>,
    pub config: Arc<ConfigurationService>,
    pub instance: Arc<InstanceService>,
    pub totp: Arc<TotpService>,
    pub file: Arc<FileService>,
    pub settings: Arc<SettingsService>,
    pub role: Arc<RwLock<RoleService>>,
    pub secure_cookies: bool,
}
