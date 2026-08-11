use crate::error::{AppError, AppResult};
use crate::repo::repository::ModerationRepository;
use crate::services::ModerationProvider;
use async_trait::async_trait;
use chrono::Utc;
use rumary_dto::domain::api::value_object::user::UserId;
use rumary_dto::domain::api::{BanId, NewUserBan, UserBan};
use rumary_dto::dto::api::request::{CreateUserBanRequest, RevokeUserBanRequest};
use std::sync::Arc;

pub struct ModerationService {
    repo: Arc<dyn ModerationRepository<Error = AppError>>,
}

impl ModerationService {
    pub fn new(repo: Arc<dyn ModerationRepository<Error = AppError>>) -> Self {
        Self { repo }
    }

    fn validate_request(
        actor_id: UserId,
        target_id: UserId,
        request: CreateUserBanRequest,
    ) -> AppResult<NewUserBan> {
        if actor_id == target_id {
            return Err(AppError::Forbidden("cannot ban yourself".to_owned()));
        }
        if !request.scope.blocks_api() {
            return Err(AppError::Validation(
                "launcher and game scopes are not supported by this API yet".to_owned(),
            ));
        }
        let reason_code = request.reason_code.trim().to_owned();
        if reason_code.is_empty() || reason_code.len() > 64 {
            return Err(AppError::Validation(
                "reason_code must contain 1..64 characters".to_owned(),
            ));
        }
        if let Some(note) = &request.staff_note
            && note.len() > 2000
        {
            return Err(AppError::Validation(
                "staff_note must contain at most 2000 characters".to_owned(),
            ));
        }
        let starts_at = request.starts_at.unwrap_or_else(Utc::now);
        if request
            .expires_at
            .is_some_and(|expires| expires <= starts_at)
        {
            return Err(AppError::Validation(
                "expires_at must be later than starts_at".to_owned(),
            ));
        }
        Ok(NewUserBan {
            user_id: target_id,
            scope: request.scope,
            starts_at,
            expires_at: request.expires_at,
            reason_code,
            staff_note: request.staff_note,
            created_by: actor_id,
        })
    }
}

#[async_trait]
impl ModerationProvider for ModerationService {
    type Error = AppError;

    async fn check_api_access(&self, user_id: UserId) -> AppResult<()> {
        if self.repo.find_active_api_ban(user_id).await?.is_some() {
            return Err(AppError::Banned("account is banned".to_owned()));
        }
        Ok(())
    }

    async fn ban_user(
        &self,
        actor_id: UserId,
        target_id: UserId,
        request: CreateUserBanRequest,
    ) -> AppResult<UserBan> {
        let ban = Self::validate_request(actor_id, target_id, request)?;
        self.repo.create_user_ban_and_revoke_sessions(ban).await
    }

    async fn list_user_bans(&self, user_id: UserId) -> AppResult<Vec<UserBan>> {
        self.repo.list_user_bans(user_id).await
    }

    async fn revoke_user_ban(
        &self,
        actor_id: UserId,
        target_id: UserId,
        ban_id: BanId,
        request: RevokeUserBanRequest,
    ) -> AppResult<UserBan> {
        let reason = request.reason.trim().to_owned();
        if reason.is_empty() || reason.len() > 500 {
            return Err(AppError::Validation(
                "reason must contain 1..500 characters".to_owned(),
            ));
        }
        self.repo
            .revoke_user_ban(ban_id, target_id, actor_id, reason)
            .await?
            .ok_or_else(|| AppError::NotFound("active ban was not found".to_owned()))
    }
}
