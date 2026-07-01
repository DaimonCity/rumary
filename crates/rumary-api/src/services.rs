use crate::service::auth::AuthenticatedUser;
use crate::service::totp::TotpService;
use crate::service::userprofile::ProfileResponse;
use async_trait::async_trait;
use rumary_dto::domain::api::{DeleteMeRequest, LoginOutcome};
use rumary_dto::dto::api::request::{LoginRequest, RegisterRequest, TotpLoginRequest};
use rumary_dto::dto::api::response::{SessionTokensResponse, WsTicketResponse};
use uuid::Uuid;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    type Error;
    async fn register(
        &self,
        payload: RegisterRequest,
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn login(&self, payload: LoginRequest) -> Result<LoginOutcome, Self::Error>; // +
    async fn verify_totp(
        &self,
        payload: TotpLoginRequest,
        totp_service: &TotpService,
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn refresh(
        &self,
        refresh_token: &str,
        refresh_token_id: Uuid,
    ) -> Result<SessionTokensResponse, Self::Error>; // +

    async fn logout(&self, auth_user: &AuthenticatedUser) -> Result<(), Self::Error>; // +

    async fn authenticate_ws_ticket(&self, ticket: &str) -> Result<AuthenticatedUser, Self::Error>; // +
    async fn issue_ws_ticket(
        &self,
        auth_user: &AuthenticatedUser,
    ) -> Result<WsTicketResponse, Self::Error>; // +
    async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedUser, Self::Error>; // only for FromRequesParts
}

#[async_trait]
pub trait UserProfileProvider: Send + Sync {
    type Error;
    async fn me(&self, user_uuid: Uuid) -> Result<ProfileResponse, Self::Error>;
    async fn delete_me(&self, user_uuid: Uuid, payload: DeleteMeRequest)
    -> Result<(), Self::Error>;
}
