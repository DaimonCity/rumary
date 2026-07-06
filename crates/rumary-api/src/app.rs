use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::repo::db::PostgresRepo;
use crate::repo::repository::{
    ConfigurationRepository, InstanceRepository, SessionRepository, SettingsRepository,
    TotpRepository, UserRepository,
};
use crate::service::api;
use crate::service::auth::AuthService;
use crate::service::file::FileService;
use crate::service::totp::TotpService;
use crate::service::userprofile::UserProfileService;
use crate::state::AppState;
use sqlx::migrate::Migrator;
use std::sync::Arc;

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

        let repository = Repositories {
            user_repo,
            totp_repo,
            configuration_repo,
            instance_repo,
            session_repo,
            settings_repo,
        };

        let state = Self::build_components(config.as_ref(), repository)?;

        Ok(Self { config, state })
    }

    pub async fn run(self) -> AppResult<()> {
        let listener = tokio::net::TcpListener::bind((self.config.host.as_str(), self.config.port))
            .await
            .map_err(AppError::Io)?;

        axum::serve(listener, api::build_router(Arc::from(self.state)))
            .await
            .map_err(AppError::Io)
    }

    async fn run_migrations(repo: &PostgresRepo) -> AppResult<()> {
        let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
        migrator.run(&repo.get_pool()).await?;
        Ok(())
    }

    fn build_components(config: &Config, repositories: Repositories) -> AppResult<AppState> {
        let auth = Arc::new(AuthService::new(
            repositories.user_repo.clone(),
            repositories.session_repo.clone(),
            repositories.totp_repo.clone(),
            config.jwt_secret.clone(),
            config.access_token_ttl_minutes,
            config.refresh_token_ttl_days,
            config.ws_ticket_ttl_seconds,
        ));

        let totp = Arc::new(TotpService::new(
            repositories.totp_repo.clone(),
            config.totp_secret_key(),
        ));

        let user_profile = Arc::new(UserProfileService::new(
            repositories.user_repo,
            repositories.totp_repo,
        ));
        let file = Arc::new(FileService::new(
            repositories.configuration_repo,
            repositories.instance_repo,
            repositories.settings_repo,
        ));

        let state = AppState {
            auth,
            user_profile,
            file,
            totp,
            secure_cookies: config.secure_cookies,
        };

        Ok(state)
    }
}
