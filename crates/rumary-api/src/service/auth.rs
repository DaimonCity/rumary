use crate::error::{AppError, AppResult};
use crate::repo::repository::{SessionRepository, TotpRepository, UserRepository};
use crate::service::totp::TotpService;
use crate::services::AuthProvider;
use crate::state::AppState;
use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use http::header;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rumary_dto::domain::api::ws::WsTicketClaims;
use rumary_dto::domain::api::{AccessLevel, LoginOutcome, User};
use rumary_dto::domain::api::{NewUser, RefreshSessionUpdate};
use rumary_dto::dto::api::request::{
    ClaimsRequest, LoginRequest, RegisterRequest, TotpLoginRequest,
};
use rumary_dto::dto::api::response::{
    ClaimsResponse, SessionTokensResponse, TotpRequiredResponse, WsTicketResponse,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<dyn UserRepository<Error = AppError>>,
    session_repo: Arc<dyn SessionRepository<Error = AppError>>,
    totp_repo: Arc<dyn TotpRepository<Error = AppError>>,
    jwt_secret: String,
    access_token_ttl_minutes: i64,
    refresh_token_ttl_days: i64,
    ws_ticket_ttl_seconds: i64,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository<Error = AppError>>,
        session_repo: Arc<dyn SessionRepository<Error = AppError>>,
        totp_repo: Arc<dyn TotpRepository<Error = AppError>>,
        jwt_secret: String,
        access_token_ttl_minutes: i64,
        refresh_token_ttl_days: i64,
        ws_ticket_ttl_seconds: i64,
    ) -> Self {
        Self {
            user_repo,
            jwt_secret,
            session_repo,
            totp_repo,
            access_token_ttl_minutes,
            refresh_token_ttl_days,
            ws_ticket_ttl_seconds,
        }
    }

    async fn issue_tokens(&self, user_uuid: Uuid) -> AppResult<SessionTokensResponse> {
        let refresh_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let refresh_token_hash = hash(&refresh_token, DEFAULT_COST)
            .map_err(|_| AppError::Crypto("failed to hash refresh token".to_string()))?;
        let refresh_token_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(self.refresh_token_ttl_days);

        self.session_repo
            .save_refresh_session(
                user_uuid,
                RefreshSessionUpdate {
                    token_id: refresh_token_id,
                    refresh_token_hash,
                    expires_at,
                },
            )
            .await?;

        let refreshed_user =
            self.user_repo
                .find_user(user_uuid)
                .await?
                .ok_or(AppError::NotFound(
                    "user was not found while ws ticket".to_string(),
                ))?;

        Ok(SessionTokensResponse {
            access_token: self.encode_access_token(&refreshed_user)?,
            refresh_token,
            refresh_token_id,
        })
    }

    fn encode_access_token(&self, user: &User) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.access_token_ttl_minutes);
        let level = user.access_level.into();
        let claims = ClaimsResponse {
            sub: user.uuid.to_string(),
            level,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::Token("failed to encode access token".to_string()))
    }
}

#[async_trait]
impl AuthProvider for AuthService {
    type Error = AppError;

    async fn register(&self, payload: RegisterRequest) -> AppResult<SessionTokensResponse> {
        let password_hash = hash(payload.password, DEFAULT_COST)
            .map_err(|_| AppError::Crypto("failed to hash password".to_string()))?;

        let user = self
            .user_repo
            .create_user(NewUser {
                nickname: payload.nickname,
                login: payload.login,
                password_hash,
            })
            .await?;

        self.issue_tokens(user.uuid).await
    }

    async fn login(&self, payload: LoginRequest) -> AppResult<LoginOutcome> {
        let user = self
            .user_repo
            .find_user_by_login(&payload.login)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while logging".to_string(),
            ))?;

        let is_valid = verify(payload.password, &user.password_hash)
            .map_err(|_| AppError::Crypto("failed to verify password".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid credentials".to_string()));
        }

        let res = self.totp_repo.find_totp_user(user.uuid).await?;

        if res.is_some() {
            return Ok(LoginOutcome::TotpRequired(TotpRequiredResponse {
                user_uuid: user.uuid,
            }));
        }

        self.issue_tokens(user.uuid).await.map(LoginOutcome::Tokens)
    }

    async fn verify_totp(
        &self,
        payload: TotpLoginRequest,
        totp_service: &TotpService,
    ) -> AppResult<SessionTokensResponse> {
        let user = self
            .totp_repo
            .find_totp_user(payload.user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "totp user was not found while logging".to_string(),
            ))?;

        totp_service.verify_user_code(&user, &payload.totp_code)?;
        self.issue_tokens(user.uuid).await
    }

    async fn refresh(
        &self,
        refresh_token: &str,
        refresh_token_id: Uuid,
    ) -> AppResult<SessionTokensResponse> {
        let user = self
            .session_repo
            .find_user_by_token_id(refresh_token_id)
            .await?
            .ok_or(AppError::NotFound(
                "totp user was not found while logging".to_string(),
            ))?;
        let expires_at = user.expires_at;

        if Utc::now() > expires_at {
            self.session_repo.clear_refresh_session(user.uuid).await?;
            return Err(AppError::Unauthorized(
                "refresh session expired".to_string(),
            ));
        }

        let stored_hash = user.refresh_token_hash;
        let is_valid = verify(refresh_token, &stored_hash)
            .map_err(|_| AppError::Crypto("failed to verify refresh token".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid refresh token".to_string()));
        }

        self.issue_tokens(user.uuid).await
    }

    async fn logout(&self, auth_user: &AuthenticatedUser) -> AppResult<()> {
        self.session_repo
            .clear_refresh_session(auth_user.uuid)
            .await?;
        Ok(())
    }

    async fn authenticate_ws_ticket(&self, ticket: &str) -> AppResult<AuthenticatedUser> {
        let claims = decode::<WsTicketClaims>(
            ticket,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid websocket ticket".to_string()))?
        .claims;

        if claims.purpose != "ws" {
            return Err(AppError::Unauthorized(
                "invalid websocket ticket".to_string(),
            ));
        }

        let user_uuid = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("invalid websocket ticket".to_string()))?;
        let user = self
            .user_repo
            .find_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found in ws ticket".to_string(),
            ))?;

        Ok(AuthenticatedUser {
            uuid: user.uuid,
            access_level: claims.level,
        })
    }

    async fn issue_ws_ticket(&self, auth_user: &AuthenticatedUser) -> AppResult<WsTicketResponse> {
        let user = self
            .user_repo
            .find_user(auth_user.uuid)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while ws ticket".to_string(),
            ))?;
        let now = Utc::now();
        let exp = now + Duration::seconds(self.ws_ticket_ttl_seconds);
        let claims = WsTicketClaims {
            sub: user.uuid.to_string(),
            level: user.access_level,
            purpose: "ws".to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let ws_ticket = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::Token("failed to encode websocket ticket".to_string()))?;

        Ok(WsTicketResponse { ws_ticket })
    }

    async fn authenticate_access_token(&self, token: &str) -> AppResult<AuthenticatedUser> {
        let claims = decode::<ClaimsRequest>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid access token".to_string()))?
        .claims;

        let user_uuid = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("invalid access token".to_string()))?;
        let user = self
            .user_repo
            .find_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while auth token".to_string(),
            ))?;

        Ok(AuthenticatedUser {
            uuid: user.uuid,
            access_level: claims.level.into(),
        })
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = Arc::<AppState>::from_ref(state);
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::to_owned);

        async move {
            let token = token.ok_or(AppError::Unauthorized("missing bearer token".to_string()))?;
            state.auth.authenticate_access_token(&token).await
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub uuid: Uuid,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone)]
pub struct VipUser;
#[derive(Debug, Clone)]
pub struct BuilderUser;
#[derive(Debug, Clone)]
pub struct WriterUser;
#[derive(Debug, Clone)]
pub struct AdminUser;
#[derive(Debug, Clone)]
pub struct OwnerUser;

#[derive(Debug, Clone)]
pub struct MaybeWorkerUser(pub Option<AuthenticatedUser>);
