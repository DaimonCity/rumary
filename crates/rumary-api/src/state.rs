use crate::error::AppError;
use crate::service::totp::TotpService;
use crate::services::{AuthProvider, UserProfileProvider};
use std::sync::Arc;
use crate::service::file::FileService;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthProvider<Error = AppError>>,
    pub user_profile: Arc<dyn UserProfileProvider<Error = AppError>>,
    pub totp: Arc<TotpService>,
    pub file: Arc<FileService>,
    pub secure_cookies: bool,
}