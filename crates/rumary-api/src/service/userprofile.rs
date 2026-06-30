use std::sync::Arc;
use bcrypt::verify;
use uuid::Uuid;
use rumary_dto::domain::api::DeleteMeRequest;
use crate::error::AppError;
use crate::repo::repository::UserRepository;

pub struct UserProfileService {
    user_repo: Arc<dyn UserRepository>
}

impl UserProfileService {
    pub(crate) fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn delete_me(
        &self,
        user_uuid: Uuid,
        payload: DeleteMeRequest,
    ) -> Result<(), AppError> {
        let user = self
            .user_repo
            .find_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while logging".to_string(),
            ))?;

        let is_valid = verify(payload.password, &user.password_hash)
            .map_err(|_| AppError::Crypto("failed to verify password".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid password".to_string()));
        }

        self.user_repo.delete_user(user_uuid).await
    }
}