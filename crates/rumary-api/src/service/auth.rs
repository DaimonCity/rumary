use crate::error::{AppError, AppResult};
use crate::repo::repository::{SessionRepository, UserRepository};
use crate::service::totp::TotpService;
use crate::services::AuthProvider;
use crate::state::AppState;
use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use http::header;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rumary_dto::domain::api::WsTicketClaims;
use rumary_dto::domain::api::{AccessLevel, LoginOutcome, RoleType, User};
use rumary_dto::domain::api::{NewUser, RefreshSessionUpdate};
use rumary_dto::domain::auth::expiration_time::ExpirationTime;
use rumary_dto::domain::user::{PasswordHash, UserId};
use rumary_dto::domain::value_object::auth::tokens::{TokenHash, TokenId};
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
    jwt_secret: String,
    access_token_ttl_minutes: i64,
    refresh_token_ttl_days: i64,
    ws_ticket_ttl_seconds: i64,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository<Error = AppError>>,
        session_repo: Arc<dyn SessionRepository<Error = AppError>>,
        jwt_secret: String,
        access_token_ttl_minutes: i64,
        refresh_token_ttl_days: i64,
        ws_ticket_ttl_seconds: i64,
    ) -> Self {
        Self {
            user_repo,
            jwt_secret,
            session_repo,
            access_token_ttl_minutes,
            refresh_token_ttl_days,
            ws_ticket_ttl_seconds,
        }
    }

    async fn issue_tokens(&self, user_id: UserId) -> AppResult<SessionTokensResponse> {
        let refresh_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let refresh_token_hash = TokenHash::new(
            hash(&refresh_token, DEFAULT_COST)
                .map_err(|_| AppError::Crypto("failed to hash refresh token".to_string()))?,
        );
        let refresh_token_id = TokenId(Uuid::new_v4());
        let expires_at =
            ExpirationTime::new(Utc::now() + Duration::days(self.refresh_token_ttl_days))?;

        self.session_repo
            .save_refresh_session(
                user_id,
                RefreshSessionUpdate {
                    token_id: refresh_token_id.clone(),
                    refresh_token_hash,
                    expires_at,
                },
            )
            .await?;

        let refreshed_user = self
            .user_repo
            .find_user(user_id)
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
            sub: user.id.to_string(),
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
        let password_hash = PasswordHash::new(payload.password)?;
        let user = self
            .user_repo
            .create_user(NewUser {
                nickname: payload.nickname.try_into()?,
                login: payload.login.try_into()?,
                password_hash,
            })
            .await?;

        self.issue_tokens(user.id).await
    }

    async fn login(
        &self,
        payload: LoginRequest,
        totp_service: &TotpService,
    ) -> AppResult<LoginOutcome> {
        let user = self
            .user_repo
            .find_user_by_login(&payload.login)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while logging".to_string(),
            ))?;

        let is_valid = user.password_hash.verify(&payload.password);

        if let Ok(is_valid) = is_valid
            && is_valid
        {
        } else {
            return Err(AppError::Unauthorized("invalid credentials".to_string()));
        }

        let res = totp_service.is_enabled(user.id).await?;

        if res {
            return Ok(LoginOutcome::TotpRequired(TotpRequiredResponse {
                user_id: user.id.into(), // Можно ли раскрыть uuid пользователя? Или сделать типа PubUserId?
            }));
        }

        self.issue_tokens(user.id).await.map(LoginOutcome::Tokens)
    }

    async fn verify_totp(
        &self,
        payload: TotpLoginRequest,
        totp_service: &TotpService,
    ) -> AppResult<SessionTokensResponse> {
        let user_id = UserId(payload.user_id);

        if !totp_service.is_enabled(user_id).await?
            || !totp_service
                .verify_user_code(user_id, &payload.totp_code)
                .await?
        {
            return Err(AppError::NotFound(format!(
                "totp user with id {} not found",
                user_id
            )));
        }

        self.issue_tokens(user_id).await
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

        let user_id = user.id.into();
        if Utc::now() > expires_at {
            self.session_repo.clear_refresh_session(user_id).await?;
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

        self.issue_tokens(user_id).await
    }

    async fn logout(&self, auth_user: &AuthenticatedUser) -> AppResult<()> {
        self.session_repo
            .clear_refresh_session(auth_user.id)
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

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("invalid websocket ticket".to_string()))?
            .into();
        let user = self
            .user_repo
            .find_user(user_id)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found in ws ticket".to_string(),
            ))?;

        Ok(AuthenticatedUser {
            id: user.id,
            access_level: claims.level,
        })
    }

    async fn issue_ws_ticket(&self, auth_user: &AuthenticatedUser) -> AppResult<WsTicketResponse> {
        let user = self
            .user_repo
            .find_user(auth_user.id)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while ws ticket".to_string(),
            ))?;
        let now = Utc::now();
        let exp = now + Duration::seconds(self.ws_ticket_ttl_seconds);
        let claims = WsTicketClaims {
            sub: user.id.to_string(),
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
            .map_err(|_| AppError::Unauthorized("invalid access token".to_string()))?
            .into();
        let user = self
            .user_repo
            .find_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "user was not found while auth token".to_string(),
            ))?;

        Ok(AuthenticatedUser {
            id: user.id,
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

impl<S> FromRequestParts<S> for MaybeWorkerUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let has_header = parts.headers.contains_key(header::AUTHORIZATION);
        async move {
            if !has_header {
                return Ok(Self(None));
            }

            let future = AuthenticatedUser::from_request_parts(parts, state);
            let user = future.await?;
            match user.access_level.role_type {
                RoleType::Worker | RoleType::Owner | RoleType::VipUser | RoleType::User => {
                    Ok(Self(Some(user)))
                }
            }
        }
    }
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let future = AuthenticatedUser::from_request_parts(parts, state);
        async move {
            let user = future.await?;
            let role = match user.access_level.role_type {
                RoleType::Worker | RoleType::Owner => {
                    Self(user.access_level.role_type, user.access_level.level)
                }
                RoleType::User | RoleType::VipUser => {
                    return Err(AppError::Forbidden("admin access required".to_string()));
                }
            };

            Ok(role)
        }
    }
}
impl<S> FromRequestParts<S> for OwnerUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let future = AuthenticatedUser::from_request_parts(parts, state);
        async move {
            let user = future.await?;
            let role = match user.access_level.role_type {
                RoleType::Owner => Self,
                RoleType::Worker | RoleType::User | RoleType::VipUser => {
                    return Err(AppError::Forbidden("admin access required".to_string()));
                }
            };

            Ok(role)
        }
    }
}

/// Any user
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: UserId,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone)]
pub struct VipUser;

#[derive(Debug, Clone)]
pub struct AdminUser(pub RoleType, pub u16);

#[derive(Debug, Clone)]
pub struct OwnerUser;

#[derive(Debug, Clone)]
pub struct MaybeWorkerUser(pub Option<AuthenticatedUser>);
