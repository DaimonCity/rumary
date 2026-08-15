use crate::error::AppResult;
use crate::service::auth::AuthenticatedUser;
use crate::service::file::FileHandle;
use crate::service::totp::TotpService;
use async_trait::async_trait;
use rumary_dto::domain::api::value_object::auth::tokens::TokenId;
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::domain::api::value_object::user::UserId;
use rumary_dto::domain::api::{BanId, LoginOutcome, User, UserBan, UserSession};
use rumary_dto::dto::api::request::{
    CreateUserBanRequest, LoginRequest, RegisterRequest, RevokeUserBanRequest,
    TotpLoginRequest,
};
use rumary_dto::dto::api::response::{SessionTokensResponse, WsTicketResponse};
use std::path::Path;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Error;
    async fn register(
        &self,
        payload: RegisterRequest,
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn login(
        &self,
        payload: LoginRequest,
        totp_service: &TotpService,
    ) -> Result<LoginOutcome, Self::Error>; // +
    async fn verify_totp(
        &self,
        payload: TotpLoginRequest,
        totp_service: &TotpService,
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn refresh(
        &self,
        refresh_token: &str,
        user_session: UserSession
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn logout(&self, user_id: UserId) -> Result<(), Self::Error>; // +

    async fn authenticate_ws_ticket(&self, ticket: &str) -> Result<AuthenticatedUser, Self::Error>; // +
    async fn issue_ws_ticket(&self, user_id: UserId) -> Result<WsTicketResponse, Self::Error>; // +
    async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedUser, Self::Error>; // only for FromRequesParts

    async fn get_user_session(&self, refresh_token_id: TokenId) -> AppResult<UserSession>;
}

#[async_trait]
pub trait ModerationProvider: Send + Sync {
    type Error;

    async fn check_api_access(&self, user_id: UserId) -> Result<(), Self::Error>;
    async fn ban_user(
        &self,
        actor_id: UserId,
        target_id: UserId,
        request: CreateUserBanRequest,
    ) -> Result<UserBan, Self::Error>;
    async fn list_user_bans(&self, user_id: UserId) -> Result<Vec<UserBan>, Self::Error>;
    async fn revoke_user_ban(
        &self,
        actor_id: UserId,
        target_id: UserId,
        ban_id: BanId,
        request: RevokeUserBanRequest,
    ) -> Result<UserBan, Self::Error>;
}

#[async_trait]
pub trait UserProfileProvider: Send + Sync {
    type Error;
    async fn get(&self, user_id: UserId) -> Result<User, Self::Error>;
    
    async fn delete(&self, user_id: UserId) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait FileResolver: Send + Sync {
    // Метод возвращает не просто PathBuf, а некий абстрактный FileHandle
    // который сервис может использовать для открытия потока.
    async fn resolve_file(
        &self,
        config_uuid: ConfigurationId,
        requested_path: &Path,
        // access_level: u16,
    ) -> AppResult<FileHandle>;
}
