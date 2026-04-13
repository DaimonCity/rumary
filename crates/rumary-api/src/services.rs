use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{AuthSource, Session, SkinServiceConfig, User},
    repository::AppRepository,
};

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn register(
        &self,
        repository: Arc<dyn AppRepository>,
        username: &str,
        password: &str,
    ) -> AppResult<User>;
    async fn login(
        &self,
        repository: Arc<dyn AppRepository>,
        username: &str,
        password: &str,
    ) -> AppResult<Session>;
}

#[async_trait]
pub trait MinecraftProvider: Send + Sync {
    async fn authlib_config(&self) -> AppResult<crate::models::AuthlibConfig>;
}

#[async_trait]
pub trait SkinService: Send + Sync {
    async fn get_config(&self) -> AppResult<SkinServiceConfig>;
    async fn set_base_url(&self, base_url: Option<String>) -> AppResult<SkinServiceConfig>;
}

#[derive(Default)]
pub struct LocalAuthProvider;

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    async fn register(
        &self,
        repository: Arc<dyn AppRepository>,
        username: &str,
        password: &str,
    ) -> AppResult<User> {
        if repository.find_user_by_username(username).await?.is_some() {
            return Err(AppError::Conflict(format!(
                "user `{username}` already exists"
            )));
        }

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_owned(),
            password_hash: hash_password(password)?,
            auth_source: AuthSource::Local,
            banned: false,
            created_at: Utc::now(),
        };

        repository.insert_user(&user).await?;
        Ok(user)
    }

    async fn login(
        &self,
        repository: Arc<dyn AppRepository>,
        username: &str,
        password: &str,
    ) -> AppResult<Session> {
        let user = repository
            .find_user_by_username(username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

        if user.banned {
            return Err(AppError::Unauthorized("user is banned".into()));
        }

        verify_password(password, &user.password_hash)?;

        let session = Session {
            token: Uuid::new_v4().to_string(),
            user_id: user.id,
            issued_at: Utc::now(),
        };
        repository.insert_session(&session).await?;
        Ok(session)
    }
}

#[derive(Default)]
pub struct StubMinecraftProvider;

#[async_trait]
impl MinecraftProvider for StubMinecraftProvider {
    async fn authlib_config(&self) -> AppResult<crate::models::AuthlibConfig> {
        Ok(crate::models::AuthlibConfig {
            auth_server_url: "https://auth.example.local/api".into(),
            session_server_url: "https://session.example.local/api".into(),
            services_server_url: "https://services.example.local/api".into(),
        })
    }
}

#[derive(Default)]
pub struct StubSkinService {
    config: Arc<tokio::sync::RwLock<SkinServiceConfig>>,
}

#[async_trait]
impl SkinService for StubSkinService {
    async fn get_config(&self) -> AppResult<SkinServiceConfig> {
        Ok(self.config.read().await.clone())
    }

    async fn set_base_url(&self, base_url: Option<String>) -> AppResult<SkinServiceConfig> {
        let mut config = self.config.write().await;
        config.base_url = base_url;
        Ok(config.clone())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Internal(format!("password hashing failed: {err}")))
}

fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|err| AppError::Internal(format!("stored password hash is invalid: {err}")))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))
}
