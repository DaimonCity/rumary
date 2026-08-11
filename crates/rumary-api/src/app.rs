use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::repo::db::PostgresRepo;
use crate::repo::repository::{
    ConfigurationRepository, InstanceRepository, SessionRepository, SettingsRepository,
    TotpRepository, UserRepository, ModerationRepository,
};
use crate::service::api;
use crate::service::auth::AuthService;
use crate::service::configurations::ConfigurationService;
use crate::service::file::{FileService, LocalFileResolver};
use crate::service::group_read::GroupsReadFacade;
use crate::service::instances::InstanceService;
use crate::service::moderation::ModerationService;
use crate::service::permissions::{PermissionsFacade, ResourceTypes};
use crate::service::permissions_admin::PermissionsAdminFacade;
use crate::service::settings::SettingsService;
use crate::service::totp::TotpService;
use crate::service::userprofile::UserProfileService;
use crate::services::FileResolver;
use crate::state::AppState;
use rumary_perms::{
    GroupDirectory, PermissionAdmin, PermissionService, PermissionStore, ResourceAclStore,
};
use sqlx::migrate::Migrator;
use std::sync::Arc;
use std::time::Duration;

/// TTL кэша эффективных нод. 60 секунд — компромисс LuckPerms: изменение прав
/// вступает в силу почти сразу, но резолвинг графа групп не выполняется
/// на каждый запрос. Явная инвалидация (`invalidate_user`/`invalidate_all`)
/// после админ-операций делает задержку неощутимой.
const PERMISSION_CACHE_TTL: Duration = Duration::from_secs(60);

pub struct Application {
    config: Arc<Config>,
    state: AppState,
}

pub struct Repositories {
    user_repo: Arc<dyn UserRepository<Error = AppError>>,
    totp_repo: Arc<dyn TotpRepository<Error = AppError>>,
    configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
    session_repo: Arc<dyn SessionRepository<Error = AppError>>,
    settings_repo: Arc<dyn SettingsRepository<Error = AppError>>,
    perms_repo: Arc<dyn PermissionStore>,
    resource_acl_repo: Arc<dyn ResourceAclStore>,
    perms_admin_repo: Arc<dyn PermissionAdmin>,
    group_dir_repo: Arc<dyn GroupDirectory>,
    moderation_repo: Arc<dyn ModerationRepository<Error = AppError>>,
}

impl Application {
    pub async fn build(config: Config) -> AppResult<Self> {
        let config = Arc::new(config);
        let repo = Arc::new(PostgresRepo::connect(config.database.clone()).await?);
        Self::run_migrations(&repo).await?;

        let user_repo: Arc<dyn UserRepository<Error = AppError>> = repo.clone();
        let totp_repo: Arc<dyn TotpRepository<Error = AppError>> = repo.clone();
        let configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>> = repo.clone();
        let instance_repo: Arc<dyn InstanceRepository<Error = AppError>> = repo.clone();
        let session_repo: Arc<dyn SessionRepository<Error = AppError>> = repo.clone();
        let settings_repo: Arc<dyn SettingsRepository<Error = AppError>> = repo.clone();
        let perms_repo: Arc<dyn PermissionStore> = repo.clone();
        let resource_acl_repo: Arc<dyn ResourceAclStore> = repo.clone();
        let perms_admin_repo: Arc<dyn PermissionAdmin> = repo.clone();
        let group_dir_repo: Arc<dyn GroupDirectory> = repo.clone();
        let moderation_repo: Arc<dyn ModerationRepository<Error = AppError>> = repo.clone();

        let repository = Repositories {
            user_repo,
            totp_repo,
            configuration_repo,
            instance_repo,
            session_repo,
            settings_repo,
            perms_repo,
            resource_acl_repo,
            perms_admin_repo,
            group_dir_repo,
            moderation_repo,
        };

        let state = Self::build_components(config.as_ref(), repository, config.first_time).await?;

        Ok(Self { config, state })
    }

    pub async fn run(self) -> AppResult<()> {
        let listener = tokio::net::TcpListener::bind((self.config.host.as_str(), self.config.port))
            .await
            .map_err(AppError::Io)?;

        axum::serve(listener, api::build_router(Arc::from(self.state)))
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }

    async fn run_migrations(repo: &PostgresRepo) -> AppResult<()> {
        let migrator = Migrator::new(std::path::Path::new("./crates/rumary-api/migrations")).await?;
        migrator.run(&repo.get_pool()).await?;
        Ok(())
    }

    async fn build_components(
        config: &Config,
        repositories: Repositories,
        _first_time: bool,
    ) -> AppResult<AppState> {
        let moderation: Arc<dyn crate::services::ModerationProvider<Error = AppError>> =
            Arc::new(ModerationService::new(repositories.moderation_repo));
        let auth = Arc::new(AuthService::new(
            repositories.user_repo.clone(),
            repositories.session_repo.clone(),
            config.jwt_secret.clone(),
            config.access_token_ttl_minutes,
            config.refresh_token_ttl_days,
            config.ws_ticket_ttl_seconds,
            moderation.clone(),
        ));

        let local = LocalFileResolver::new(
            repositories.configuration_repo.clone(),
            repositories.instance_repo.clone(),
            repositories.settings_repo.clone(),
        );

        let totp = Arc::new(TotpService::new(
            repositories.totp_repo.clone(),
            config.totp_secret_key(),
        ));

        let user_profile = Arc::new(UserProfileService::new(
            repositories.user_repo,
            repositories.totp_repo,
        ));

        let resolver: Arc<dyn FileResolver> = Arc::from(local);

        let file = Arc::new(FileService::new(resolver));

        let settings = Arc::new(SettingsService::new(repositories.settings_repo));

        let config_service = Arc::new(ConfigurationService::new(repositories.configuration_repo));
        let instance_service = Arc::new(InstanceService::new(repositories.instance_repo));

        let perm_service = Arc::new(PermissionService::from_arc(
            repositories.perms_repo,
            PERMISSION_CACHE_TTL,
        ));
        let group_read = Arc::new(GroupsReadFacade::new(repositories.group_dir_repo.clone()));

        let perms = Arc::new(PermissionsFacade::from_arc(
            perm_service.clone(),
            repositories.resource_acl_repo.clone(),
        ));

        let perms_admin = Arc::new(PermissionsAdminFacade::from_arc(
            perm_service.clone(),
            repositories.perms_admin_repo,
            group_read.clone(),
        ));

        let state = AppState {
            auth,
            user_profile,
            moderation,
            config: config_service,
            instance: instance_service,
            file,
            totp,
            settings,
            perms,
            perms_admin,
            group_read,
            resource_types: Arc::new(ResourceTypes::new()?),
            secure_cookies: config.secure_cookies,
        };

        Ok(state)
    }
}
