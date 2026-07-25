use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::repo::db::PostgresRepo;
use crate::repo::repository::{
    ConfigurationRepository, InstanceRepository, RightsRepository, RolesRepository,
    SessionRepository, SettingsRepository, TotpRepository, UserRepository,
};
use crate::service::api;
use crate::service::auth::AuthService;
use crate::service::file::{FileService, LocalFileResolver};
use crate::service::right::Rights;
use crate::service::roles::RoleService;
use crate::service::settings::SettingsService;
use crate::service::totp::TotpService;
use crate::service::userprofile::UserProfileService;
use crate::services::FileResolver;
use crate::state::AppState;
use sqlx::migrate::Migrator;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::service::configurations::ConfigurationService;
use crate::service::instances::InstanceService;

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
    roles_repo: Arc<dyn RolesRepository<Error = AppError>>,
    rights_repo: Arc<dyn RightsRepository<Error = AppError>>,
}

impl Application {
    pub async fn build(config: Config) -> AppResult<Self> {
        let config = Arc::new(config);
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let repo = Arc::new(PostgresRepo::connect(config.database.clone(), tx).await?);
        Self::run_migrations(&repo).await?;

        let user_repo: Arc<dyn UserRepository<Error = AppError>> = repo.clone();
        let totp_repo: Arc<dyn TotpRepository<Error = AppError>> = repo.clone();
        let configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>> = repo.clone();
        let instance_repo: Arc<dyn InstanceRepository<Error = AppError>> = repo.clone();
        let session_repo: Arc<dyn SessionRepository<Error = AppError>> = repo.clone();
        let settings_repo: Arc<dyn SettingsRepository<Error = AppError>> = repo.clone();
        let roles_repo: Arc<dyn RolesRepository<Error = AppError>> = repo.clone();
        let rights_repo: Arc<dyn RightsRepository<Error = AppError>> = repo.clone();

        let repository = Repositories {
            user_repo,
            totp_repo,
            configuration_repo,
            instance_repo,
            session_repo,
            settings_repo,
            roles_repo,
            rights_repo,
        };

        let state = Self::build_components(config.as_ref(), repository, rx).await?;

        Ok(Self { config, state })
    }

    pub async fn run(self) -> AppResult<()> {
        let role = self.state.role.clone();
        let rights_updates = {
            let mut role_service = role.write().await;
            role_service.take_rights_channel()
        };

        if let Some(mut rights_updates) = rights_updates {
            tokio::spawn(async move {
                while let Some(new_rights) = rights_updates.recv().await {
                    role.write().await.apply_rights(new_rights).await;
                }
            });
        }

        let listener = tokio::net::TcpListener::bind((self.config.host.as_str(), self.config.port))
            .await
            .map_err(AppError::Io)?;

        axum::serve(listener, api::build_router(Arc::from(self.state)))
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }

    async fn run_migrations(repo: &PostgresRepo) -> AppResult<()> {
        let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
        migrator.run(&repo.get_pool()).await?;
        Ok(())
    }

    async fn build_components(
        config: &Config,
        repositories: Repositories,
        rx: tokio::sync::mpsc::Receiver<Rights>,
    ) -> AppResult<AppState> {
        let auth = Arc::new(AuthService::new(
            repositories.user_repo.clone(),
            repositories.session_repo.clone(),
            config.jwt_secret.clone(),
            config.access_token_ttl_minutes,
            config.refresh_token_ttl_days,
            config.ws_ticket_ttl_seconds,
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

        let role = Arc::new(RwLock::new(
            RoleService::new(repositories.roles_repo, repositories.rights_repo, rx).await?,
        ));

        let config_service = Arc::new(ConfigurationService::new(repositories.configuration_repo));
        let instance_service = Arc::new(InstanceService::new(repositories.instance_repo));

        let state = AppState {
            auth,
            user_profile,
            config: config_service,
            instance: instance_service,
            file,
            totp,
            role,
            settings,
            secure_cookies: config.secure_cookies,
        };

        Ok(state)
    }
}
