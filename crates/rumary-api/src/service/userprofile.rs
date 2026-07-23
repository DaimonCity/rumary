use crate::error::{AppError, AppResult};
use crate::repo::repository::{TotpRepository, UserRepository};
use crate::services::UserProfileProvider;
use async_trait::async_trait;
use bcrypt::verify;
use rumary_dto::domain::user::UserId;
use rumary_dto::dto::api::request::DeleteMeRequest;
use serde::Serialize;
use std::sync::Arc;
use rumary_dto::domain::api::{RoleId, User};

pub struct UserProfileService {
    user_repo: Arc<dyn UserRepository<Error=AppError>>,
    totp_repo: Arc<dyn TotpRepository<Error=AppError>>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub login: String,
    pub nickname: String,
    pub has_totp: bool,
}

impl UserProfileService {
    pub(crate) fn new(
        user_repo: Arc<dyn UserRepository<Error=AppError>>,
        totp_repo: Arc<dyn TotpRepository<Error=AppError>>,
    ) -> Self {
        Self {
            user_repo,
            totp_repo,
        }
    }

    async fn get_user(&self, user_id: UserId) -> Result<User, AppError> {
        Ok(self
            .user_repo
            .find_user(user_id)
            .await?
            .ok_or(AppError::NotFound(
                "UserProfileService: User not found".to_owned(),
            ))?)
    }
}

#[async_trait]
impl UserProfileProvider for UserProfileService {
    type Error = AppError;
    async fn me(&self, user_id: UserId) -> AppResult<ProfileResponse> {
        let user = self.get_user(user_id).await?;

        let totp_user = self.totp_repo.find_totp_user(user_id).await?;

        let profile = ProfileResponse {
            login: user.login.into(),
            nickname: user.nickname.into(),
            has_totp: totp_user.is_some(),
        };

        Ok(profile)
    }

    async fn users_roles(&self, user_id: UserId) -> Result<Vec<RoleId>, Self::Error> {
        let user = self.get_user(user_id).await?;

        Ok(user.roles)
    }

    async fn delete_me(&self, user_id: UserId, payload: DeleteMeRequest) -> AppResult<()> {
        let user = self.get_user(user_id).await?;

        let is_valid = verify(payload.password, &user.password_hash)
            .map_err(|_| AppError::Crypto("failed to verify password".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid password".to_string()));
        }

        self.user_repo.delete_user(user_id).await
    }
}
