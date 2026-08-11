use crate::error::AppError;
use crate::service::configurations::ConfigurationService;
use crate::service::file::FileService;
use crate::service::instances::InstanceService;
use crate::service::settings::SettingsService;
use crate::service::totp::TotpService;
use crate::service::permissions::{ResourceTypes, SharedPermissions};
use crate::services::{AuthProvider, ModerationProvider, UserProfileProvider};
use std::sync::Arc;
use crate::service::group_read::GroupsReadFacade;
use crate::service::permissions_admin::PermissionsAdminFacade;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthProvider<Error = AppError>>,
    pub user_profile: Arc<dyn UserProfileProvider<Error = AppError>>,
    pub moderation: Arc<dyn ModerationProvider<Error = AppError>>,
    pub config: Arc<ConfigurationService>,
    pub instance: Arc<InstanceService>,
    pub totp: Arc<TotpService>,
    pub file: Arc<FileService>,
    pub settings: Arc<SettingsService>,
    /// Проверка прав: RBAC + resource-level ACL + ранг.
    pub perms: SharedPermissions,
    pub perms_admin: Arc<PermissionsAdminFacade>,
    pub group_read: Arc<GroupsReadFacade>,
    // pub group_dir: Arc<dyn GroupDirectory>,
    /// Провалидированные при старте типы ресурсов для сборки ключей прав.
    pub resource_types: Arc<ResourceTypes>,
    pub secure_cookies: bool,
}
