use crate::error::AppResult;
use async_trait::async_trait;
use rumary_dto::domain::api::{NewTotpUser, NewUser, RefreshSessionUpdate, TotpUser, User, UserSession};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: NewUser) -> AppResult<User>;
    async fn find_user(&self, uuid: Uuid) -> AppResult<Option<User>>;
    async fn find_user_by_login(&self, login: &str) -> AppResult<Option<User>>;
    async fn delete_user(&self, uuid: Uuid) -> AppResult<()>;
    async fn users_list(&self) -> AppResult<Vec<User>>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_user_by_token_id(&self,
                                   token_uuid: Uuid) -> AppResult<Option<UserSession>>;
    async fn save_refresh_session(
        &self,
        user_uuid: Uuid,
        session: RefreshSessionUpdate,
    ) -> AppResult<()>;
    async fn clear_refresh_session(&self, user_uuid: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait TotpRepository: Send + Sync {
    async fn create_totp_user(&self, user: NewTotpUser) -> AppResult<TotpUser>;
    async fn totp_user_confirmed(&self, uuid: Uuid) -> AppResult<TotpUser>;
    async fn find_totp_user(&self, uuid: Uuid) -> AppResult<Option<TotpUser>>;
    async fn delete_totp_user(&self, uuid: Uuid) -> AppResult<()>;
}
