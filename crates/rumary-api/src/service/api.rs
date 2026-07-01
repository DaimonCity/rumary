use crate::service::auth::{AuthenticatedUser, MaybeWorkerUser};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::response::{IntoResponse, Response};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};use rumary_dto::domain::api::{DeleteMeRequest, LoginOutcome, RoleType};
use rumary_dto::dto::api::request::{LoginRequest, RegisterRequest, TotpLoginRequest};
use rumary_dto::dto::api::response::{SessionTokensResponse, TokenResponse};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use crate::service::userprofile::ProfileResponse;

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const REFRESH_TOKEN_ID_COOKIE: &str = "refresh_token_id";

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/login/totp", post(verify_totp))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/users/me", get(get_me).delete(delete_me))
        // .route("/api/v1/auth/ws-ticket", post(issue_ws_ticket))
        // .route("/api/users/{user_id}/ban", post(ban_user))
        // .route("/api/users/{user_id}/unban", post(unban_user))
        .with_state(state)
    // .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn register(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(CookieJar, Json<TokenResponse>)> {
    let tokens = state.auth.register(payload).await?;
    Ok((
        with_session_cookies(jar, &state, &tokens),
        Json(TokenResponse {
            access_token: tokens.access_token,
        }),
    ))
}

async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {
    match state.auth.login(payload).await? {
        LoginOutcome::Tokens(tokens) => Ok((
            with_session_cookies(jar, &state, &tokens),
            Json(TokenResponse {
                access_token: tokens.access_token,
            }),
        )
            .into_response()),
        LoginOutcome::TotpRequired(response) => {
            Ok((http::StatusCode::ACCEPTED, Json(response)).into_response())
        }
    }
}

async fn verify_totp(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<TotpLoginRequest>,
) -> AppResult<(CookieJar, Json<TokenResponse>)> {
    let tokens = state.auth.verify_totp(payload, &state.totp).await?;
    Ok((
        with_session_cookies(jar, &state, &tokens),
        Json(TokenResponse {
            access_token: tokens.access_token,
        }),
    ))
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> AppResult<(CookieJar, Json<TokenResponse>)> {
    let refresh_token = jar
        .get(REFRESH_TOKEN_COOKIE)
        .map(|cookie| cookie.value().to_string())
        .ok_or(AppError::Unauthorized("missing refresh token".to_string()))?;
    let refresh_token_id = jar
        .get(REFRESH_TOKEN_ID_COOKIE)
        .and_then(|cookie| Uuid::parse_str(cookie.value()).ok())
        .ok_or(AppError::Unauthorized("missing refresh token id".to_string()))?;

    let tokens = state.auth.refresh(&refresh_token, refresh_token_id).await?;
    Ok((
        with_session_cookies(jar, &state, &tokens),
        Json(TokenResponse {
            access_token: tokens.access_token,
        }),
    ))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
) -> AppResult<CookieJar> {
    state.auth.logout(&auth_user).await?;
    Ok(clear_session_cookies(jar, &state))
}

async fn get_me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<ProfileResponse>> {
    let profile = state.user_profile.me(auth_user.uuid).await?;
    Ok(Json(profile))
}

async fn delete_me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
    Json(payload): Json<DeleteMeRequest>,
) -> AppResult<CookieJar> {
    state.user_profile.delete_me(auth_user.uuid, payload).await?;
    Ok(clear_session_cookies(jar, &state))
}

fn with_session_cookies(
    jar: CookieJar,
    state: &AppState,
    tokens: &SessionTokensResponse,
) -> CookieJar {
    let refresh_token_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, tokens.refresh_token.clone()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(state.secure_cookies)
        .path("/")
        .build();

    let refresh_token_id_cookie =
        Cookie::build((REFRESH_TOKEN_ID_COOKIE, tokens.refresh_token_id.to_string()))
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(state.secure_cookies)
            .path("/")
            .build();

    jar.add(refresh_token_cookie).add(refresh_token_id_cookie)
}

fn clear_session_cookies(jar: CookieJar, state: &AppState) -> CookieJar {
    let refresh_token_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(state.secure_cookies)
        .path("/")
        .build();

    let refresh_token_id_cookie = Cookie::build((REFRESH_TOKEN_ID_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(state.secure_cookies)
        .path("/")
        .build();

    jar.remove(refresh_token_cookie)
        .remove(refresh_token_id_cookie)
}

fn _include_unavailable(maybe_user: MaybeWorkerUser, level: u8) -> bool {
    maybe_user
        .0
        .map(|user| match user.access_level.role_type {
            RoleType::User | RoleType::VipUser => false,
            RoleType::Builder | RoleType::Writer | RoleType::Admin | RoleType::Owner => {
                user.access_level.level >= level
            }
        })
        .unwrap_or(false)
}