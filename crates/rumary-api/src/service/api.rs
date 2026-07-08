use crate::error::{AppError, AppResult};
use crate::service::auth::{AdminUser, AuthenticatedUser, MaybeWorkerUser};
use crate::service::userprofile::ProfileResponse;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{
    extract::State, routing::{get, post},
    Json,
    Router,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use http::HeaderMap;
use rumary_dto::domain::api::{LoginOutcome, RoleType};
use rumary_dto::dto::api::request::{DeleteMeRequest, InstancePathRequest, LoginRequest, RegisterRequest, TotpLoginRequest, UpdateConfigurationRequest, UpdateInstanceRequest};
use rumary_dto::dto::api::response::{
    GetConfigurationResponse, GetInstanceResponse, SessionTokensResponse, TokenResponse,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

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
        .route(
            "/api/v1/settings/instance_path",
            post(set_instance_path).delete(remove_instance_path),
        )
        .route(
            "/api/v1/download/{config_uuid}/{filepath}",
            get(download_file_handler),
        )
        .route("/api/v1/instance", post(create_instance))
        .route(
            "/api/v1/instance/{instance_uuid}",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .route("/api/v1/instances", get(list_instance))
        .route("/api/v1/configuration", post(create_configuration))
        .route(
            "/api/v1/configuration/{config_uuid}",
            get(get_configuration)
                .patch(update_configuration)
                .delete(delete_configuration),
        )
        .route("/api/v1/configurations", get(list_configuration))
        // .route("/api/v1/auth/ws-ticket", post(issue_ws_ticket))
        // .route("/api/users/{user_id}/ban", post(ban_user))
        // .route("/api/users/{user_id}/unban", post(unban_user))
        .with_state(state)
    // .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

///////////////////
// AUTHENTICATION
///////////////////

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
    match state.auth.login(payload, state.totp.as_ref()).await? {
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
        .ok_or(AppError::Unauthorized(
            "missing refresh token id".to_string(),
        ))?;

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

///////////////////

///////////////////
// USERS
///////////////////

async fn get_me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<ProfileResponse>> {
    let profile = state.user_profile.me(auth_user.id).await?;
    Ok(Json(profile))
}

async fn delete_me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
    Json(payload): Json<DeleteMeRequest>,
) -> AppResult<CookieJar> {
    state
        .user_profile
        .delete_me(auth_user.id, payload)
        .await?;
    Ok(clear_session_cookies(jar, &state))
}

///////////////////

///////////////////
// INSTANCE
///////////////////

/// Handler для создания instance
/// Ключ Права: instance.<config-uuid>.create
async fn create_instance(
    Path(instance_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetInstanceResponse>> {
    todo!()
}

/// Handler для получения информации о instance
/// Ключ Права: instance.<config-uuid>.get
async fn get_instance(
    Path(instance_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetInstanceResponse>> {
    todo!()
}

/// Handler для получения информации о instances
/// Ключ Права: instance.<config-uuid>.list
async fn list_instance(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetInstanceResponse>> {
    todo!()
}

/// Handler для изменения информации о instance
/// Ключ Права: instance.<config-uuid>.update
async fn update_instance(
    Path(instance_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateInstanceRequest>,
) -> AppResult<Json<GetInstanceResponse>> {
    todo!()
}

/// Handler для удаления instance
/// Ключ Права: instance.<config-uuid>.delete
async fn delete_instance(
    Path(instance_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateInstanceRequest>,
) -> AppResult<Json<GetInstanceResponse>> {
    todo!()
}

///////////////////

///////////////////
// CONFIGURATION
///////////////////

/// Handler для создания configuration
/// Ключ Права: configuration.<config-uuid>.create
async fn create_configuration(
    Path(config_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    todo!()
}

/// Handler для получения информации о configuration
/// Ключ Права: configuration.<config-uuid>.get
async fn get_configuration(
    Path(config_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetConfigurationResponse>> {
    todo!()
}

/// Handler для получения информации о configurations
/// Ключ Права: configuration.<config-uuid>.list
async fn list_configuration(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetConfigurationResponse>> {
    todo!()
}

/// Handler для изменения информации о configuration
/// Ключ Права: configuration.<config-uuid>.update
async fn update_configuration(
    Path(config_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateConfigurationRequest>,
) -> AppResult<Json<GetConfigurationResponse>> {
    todo!()
}

/// Handler для удаления configuration
/// Ключ Права: configuration.<config-uuid>.delete
async fn delete_configuration(
    Path(config_uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateConfigurationRequest>,
) -> AppResult<Json<GetConfigurationResponse>> {
    todo!()
}

/// Handler для скачивания файлов из определённой конфигурации
/// Ключ Права: configuration.<config-uuid>.download
async fn download_file_handler(
    Path((config_uuid, filepath)): Path<(Uuid, PathBuf)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    authenticated_user: AuthenticatedUser,
) -> AppResult<http::Response<Body>> {
    state
        .file
        .stream_file(
            config_uuid.into(),
            &filepath,
            &headers,
            authenticated_user.access_level.level,
        )
        .await
}

///////////////////

///////////////////
// SETTINGS
///////////////////

/// Handler для настройки пути папки с instances на сервере
/// Ключ Права: settings.instance_path.set
async fn set_instance_path(
    State(state): State<Arc<AppState>>,
    admin_user: AdminUser,
    Json(request): Json<InstancePathRequest>,
) -> AppResult<http::StatusCode> {
    match admin_user.0 {
        RoleType::User | RoleType::VipUser => Ok(http::StatusCode::FORBIDDEN),
        RoleType::Worker => {
            if admin_user.1 <= 10 {
                return Ok(http::StatusCode::FORBIDDEN);
            }
            state.settings.add_instance_path(&request.path).await?;
            Ok(http::StatusCode::CREATED)
        }
        RoleType::Owner => {
            state.settings.add_instance_path(&request.path).await?;
            Ok(http::StatusCode::CREATED)
        }
    }
}

/// Handler для настройки пути папки с instances на сервере
/// Ключ Права: settings.instance_path.remove
async fn remove_instance_path(
    State(state): State<Arc<AppState>>,
    admin_user: AdminUser,
) -> AppResult<http::StatusCode> {
    match admin_user.0 {
        RoleType::User | RoleType::VipUser => Ok(http::StatusCode::FORBIDDEN),
        RoleType::Worker => {
            if admin_user.1 <= 10 {
                // Не знаю, откуда брать это число, нужна какая-то таблица ролей
                return Ok(http::StatusCode::FORBIDDEN);
            }
            state.settings.remove_instance_path().await?;
            Ok(http::StatusCode::ACCEPTED)
        }
        RoleType::Owner => {
            state.settings.remove_instance_path().await?;
            Ok(http::StatusCode::ACCEPTED)
        }
    }
}

///////////////////

///////////////////
// UTIL FUNCTIONS FOR API
///////////////////

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

fn _include_unavailable(maybe_user: MaybeWorkerUser, level: u16) -> bool {
    maybe_user
        .0
        .map(|user| match user.access_level.role_type {
            RoleType::User | RoleType::VipUser => false,
            RoleType::Worker | RoleType::Owner => user.access_level.level >= level,
        })
        .unwrap_or(false)
}
