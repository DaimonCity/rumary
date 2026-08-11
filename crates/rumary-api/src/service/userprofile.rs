use crate::error::{AppError, AppResult};
use crate::repo::repository::{TotpRepository, UserRepository};
use crate::services::UserProfileProvider;
use async_trait::async_trait;
use bcrypt::verify;
use rumary_dto::domain::api::{User};
use rumary_dto::domain::api::value_object::user::UserId;
use std::sync::Arc;

pub struct UserProfileService {
    user_repo: Arc<dyn UserRepository<Error = AppError>>,
    totp_repo: Arc<dyn TotpRepository<Error = AppError>>,
}

impl UserProfileService {
    pub(crate) fn new(
        user_repo: Arc<dyn UserRepository<Error = AppError>>,
        totp_repo: Arc<dyn TotpRepository<Error = AppError>>,
    ) -> Self {
        Self {
            user_repo,
            totp_repo,
        }
    }

    async fn get_user(&self, user_id: UserId) -> Result<User, AppError> {
        self.user_repo
            .find_user(user_id)
            .await?
            .ok_or(AppError::NotFound(
                "UserProfileService: User not found".to_owned(),
            ))
    }
}

#[async_trait]
impl UserProfileProvider for UserProfileService {
    type Error = AppError;
    async fn get(&self, user_id: UserId) -> AppResult<User> {
        let user = self.get_user(user_id).await?;
        Ok(user)
    }

    async fn delete(&self, user_id: UserId, password: &str) -> AppResult<()> {
        let user = self.get_user(user_id).await?;

        let is_valid = verify(password, &user.password_hash)
            .map_err(|_| AppError::Crypto("failed to verify password".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid password".to_string()));
        }

        self.user_repo.delete_user(user_id).await
    }
}
