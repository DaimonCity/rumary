use crate::service::api;
use crate::service::auth::AuthService;
use crate::config::Config;
use crate::repo::db::PostgresRepo;
use crate::error::{AppError, AppResult};
use crate::repo::repository::{SessionRepository, TotpRepository, UserRepository};
use crate::state::AppState;
use crate::service::totp::TotpService;
use sqlx::migrate::Migrator;
use std::sync::Arc;
use crate::service::userprofile::UserProfileService;

pub struct Application {
    config: Arc<Config>,
    state: AppState,
}

impl Application {
    pub async fn build(config: Config) -> AppResult<Self> {
        let config = Arc::new(config);
        let repo = Arc::new(PostgresRepo::connect(config.database.clone()).await?);
        Self::run_migrations(&repo).await?;

        let user_repo: Arc<dyn UserRepository<Error=AppError>> = repo.clone();
        let totp_repo: Arc<dyn TotpRepository<Error=AppError>> = repo.clone();
        let session_repo: Arc<dyn SessionRepository<Error=AppError>> = repo.clone();
        let state = Self::build_components(config.as_ref(), user_repo, totp_repo, session_repo)?;

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

    fn build_components(
        config: &Config,
        user_repo: Arc<dyn UserRepository<Error=AppError>>,
        totp_repo: Arc<dyn TotpRepository<Error=AppError>>,
        session_repo: Arc<dyn SessionRepository<Error=AppError>>,
    ) -> AppResult<AppState> {
        let auth = Arc::new(AuthService::new(
            user_repo.clone(),
            session_repo.clone(),
            totp_repo.clone(),
            config.jwt_secret.clone(),
            config.access_token_ttl_minutes,
            config.refresh_token_ttl_days,
            config.ws_ticket_ttl_seconds,
        ));
        
        let totp = Arc::new(TotpService::new(
            totp_repo.clone(),
            config.totp_secret_key(),
        ));

        let user_profile = Arc::new(UserProfileService::new(user_repo, totp_repo));
        
        let state = AppState {
            auth,
            user_profile,
            totp,
            secure_cookies: config.secure_cookies,
        };

        Ok(state)
    }
}
