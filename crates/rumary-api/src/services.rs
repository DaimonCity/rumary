use crate::error::AppResult;
use crate::service::auth::AuthenticatedUser;
use crate::service::file::FileHandle;
use crate::service::totp::TotpService;
use crate::service::userprofile::ProfileResponse;
use async_trait::async_trait;
use rumary_dto::domain::api::{LoginOutcome, UserSession};
use rumary_dto::domain::auth::tokens::TokenId;
use rumary_dto::domain::configuration::ConfigurationId;
use rumary_dto::domain::user::UserId;
use rumary_dto::dto::api::request::{
    DeleteMeRequest, LoginRequest, RegisterRequest, TotpLoginRequest,
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
        user: UserSession
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
pub trait UserProfileProvider: Send + Sync {
    type Error;
    async fn me(&self, user_id: UserId) -> Result<ProfileResponse, Self::Error>;
    async fn delete_me(&self, user_id: UserId, payload: DeleteMeRequest)
    -> Result<(), Self::Error>;
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
