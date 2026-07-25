use crate::error::{AppError, AppResult};
use crate::service::auth::AuthenticatedUser;
// use crate::service::auth::{AdminUser, MaybeWorkerUser};
use crate::service::userprofile::ProfileResponse;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
// use rumary_dto::domain::api::RoleType;
use rumary_dto::domain::api::{LoginOutcome, RightKey, RoleId, UpdateRole};
use rumary_dto::domain::instance::InstanceId;
use rumary_dto::domain::user::UserId;
use rumary_dto::dto::api::request::{
    DeleteMeRequest, InstancePathRequest, LoginRequest, NewConfigurationRequest,
    NewInstanceRequest, RegisterRequest, TotpLoginRequest, UpdateConfigurationRequest,
    UpdateInstanceResponse,
};
use rumary_dto::dto::api::request::{NewRoleRequest, UpdateRoleRequest};
use rumary_dto::dto::api::response::role::GetRoleResponse;
use rumary_dto::dto::api::response::{
    GetConfigurationResponse, GetInstanceResponse, SessionTokensResponse,
    TokenResponse,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
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
            "/api/v1/download/{config_id}/{filepath}",
            get(download_file_handler),
        )
        .route("/api/v1/instance", post(create_instance))
        .route(
            "/api/v1/instance/{instance_id}",
            get(get_instance)
                .patch(update_instance)
                .delete(delete_instance),
        )
        .route("/api/v1/instances", get(list_instance))
        .route("/api/v1/configuration", post(create_configuration))
        .route(
            "/api/v1/configuration/{config_id}",
            get(get_configuration)
                .patch(update_configuration)
                .delete(delete_configuration),
        )
        .route("/api/v1/configurations", get(list_configuration))
        .route("/api/v1/role", post(create_role))
        .route(
            "/api/v1/role/{role_id}",
            get(get_role).patch(update_role).delete(delete_role),
        )
        .route("/api/v1/roles", get(list_roles))
        .with_state(state)
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

/// Handler для аутентификации через логин и пароль
async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {
    // action
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

/// Handler для аутентификации через totp
async fn verify_totp(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<TotpLoginRequest>,
) -> AppResult<(CookieJar, Json<TokenResponse>)> {
    // action
    let tokens = state.auth.verify_totp(payload, &state.totp).await?;
    Ok((
        with_session_cookies(jar, &state, &tokens),
        Json(TokenResponse {
            access_token: tokens.access_token,
        }),
    ))
}

/// Handler для обновления сессии по refresh_token
/// Ключ Права: auth.session.refresh
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

    let user_session = state
        .auth
        .clone()
        .get_user_session(refresh_token_id.into())
        .await?;
    let user_id = user_session.id;

    // access checking
    let is_available = check_available(state.clone(), user_id, RightKey::REFRESH).await?;

    if is_available {
        // action
        let tokens = state.auth.refresh(&refresh_token, user_session).await?;
        return Ok((
            with_session_cookies(jar, &state, &tokens),
            Json(TokenResponse {
                access_token: tokens.access_token,
            }),
        ));
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
) -> AppResult<CookieJar> {
    state.auth.logout(auth_user.id).await?;
    Ok(clear_session_cookies(jar, &state))
}

///////////////////

///////////////////
// USERS
///////////////////

/// Handler для получения информации о пользователе через access_token
/// Ключ Права: profile.<UserId>.get
async fn get_me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<ProfileResponse>> {
    let user_id = auth_user.id;
    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::get_me_key(user_id.to_string().as_str()),
    )
    .await?;

    if is_available {
        // action
        let profile = state.user_profile.me(auth_user.id).await?;
        return Ok(Json(profile));
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для получения информации о пользователе через access_token
/// Ключ Права: profile.<UserId>.delete
async fn delete_me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
    Json(payload): Json<DeleteMeRequest>,
) -> AppResult<CookieJar> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::delete_me_key(user_id.to_string().as_str()),
    )
    .await?;

    if is_available {
        // action
        state.user_profile.delete_me(auth_user.id, payload).await?;
        return Ok(clear_session_cookies(jar, &state));
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

///////////////////

///////////////////
// INSTANCE
///////////////////

/// Handler для создания instance
/// Ключ Права: instance.method.create
async fn create_instance(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewInstanceRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available =
        check_available(state.clone(), user_id, RightKey::CREATE_INSTANCE_KEY).await?;

    if is_available {
        // action
        let instance = state.instance.create_instance(payload).await?;
        let (k, v): (Vec<_>, Vec<_>) = instance.default_rights_setup().into_iter().unzip();

        let role = state.role.write().await;
        role.add_rights(&k, &v).await?;
        return Ok(http::StatusCode::OK);
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для получения информации о instance, если доступен по правам
/// Ключ Права: instance.<ConfigurationId>.get
async fn get_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetInstanceResponse>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::get_instance_key(instance_id.to_string().as_str()),
    )
    .await?;
    // action
    todo!()
}

/// Handler для получения информации о instances, доступных по праву get
/// Ключ Права: instance.method.list
async fn list_instance(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<GetInstanceResponse>>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(state.clone(), user_id, RightKey::LIST_INSTANCE_KEY).await?;
    if is_available {
        // action
        let role = state.role.read().await;
        let ids = role.instance_list(&auth_user.roles).await?;
        let configs = state.instance.list_instances(&ids).await?;

        return Ok(Json(configs));
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для изменения информации о instance
/// Ключ Права: instance.<InstanceId>.update
async fn update_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateInstanceResponse>,
) -> AppResult<Json<GetInstanceResponse>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::update_instance_key(instance_id.to_string().as_str()),
    )
    .await?;
    // action
    todo!()
}

/// Handler для удаления instance
/// Ключ Права: instance.<InstanceId>.delete
async fn delete_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::delete_instance_key(instance_id.to_string().as_str()),
    )
    .await?;
    if is_available {
        // action
        let instance = state
            .instance
            .delete_instance(InstanceId::from(instance_id))
            .await?;
        let (k, _): (Vec<_>, Vec<_>) = instance.default_rights_setup().into_iter().unzip();

        let role = state.role.write().await;
        role.remove_rights(&k).await?;
        return Ok(http::StatusCode::OK);
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

///////////////////

///////////////////
// CONFIGURATION
///////////////////

/// Handler для создания configuration
/// Ключ Права: configuration.method.create
async fn create_configuration(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewConfigurationRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available =
        check_available(state.clone(), user_id, RightKey::CREATE_CONFIGURATION_KEY).await?;
    if is_available {
        // action
        let config = state.config.create_configuration(payload).await?;
        let (k, v): (Vec<_>, Vec<_>) = config.default_rights_setup().into_iter().unzip();

        let role = state.role.write().await;
        role.add_rights(&k, &v).await?;
        return Ok(http::StatusCode::OK);
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для получения информации о configuration
/// Ключ Права: configuration.<ConfigurationId>.get
async fn get_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetConfigurationResponse>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::get_configuration_key(config_id.to_string().as_str()),
    )
    .await?;
    // action
    todo!()
}

/// Handler для получения информации о configurations, доступных по праву get
/// Ключ Права: configuration.<InstanceId>.list
async fn list_configuration(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<GetConfigurationResponse>>> {
    let user_id = auth_user.id;

    // access checking
    let is_available =
        check_available(state.clone(), user_id, RightKey::LIST_CONFIGURATION_KEY).await?;
    if is_available {
        // action
        let role = state.role.read().await;
        let ids = role.configuration_list(&auth_user.roles).await?;
        let configs = state.config.list_configs(&ids).await?;

        return Ok(Json(configs));
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для изменения информации о configuration
/// Ключ Права: configuration.<ConfigurationId>.update
async fn update_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateConfigurationRequest>,
) -> AppResult<Json<GetConfigurationResponse>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::update_configuration_key(config_id.to_string().as_str()),
    )
    .await?;
    // action
    todo!()
}

/// Handler для удаления configuration
/// Ключ Права: configuration.<ConfigurationId>.delete
async fn delete_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::delete_configuration_key(config_id.to_string().as_str()),
    )
    .await?;
    if is_available {
        // action
        let config = state.config.delete_configuration(config_id.into()).await?;
        let (k, _): (Vec<_>, Vec<_>) = config.default_rights_setup().into_iter().unzip();

        let role = state.role.write().await;
        role.remove_rights(&k).await?;
        return Ok(http::StatusCode::OK);
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для скачивания файлов из определённой конфигурации
/// Ключ Права: configuration.<ConfigurationId>.download
async fn download_file_handler(
    Path((config_id, filepath)): Path<(Uuid, PathBuf)>,
    headers: http::HeaderMap,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::Response<Body>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::download_configuration_key(config_id.to_string().as_str()),
    )
    .await?;
    // action
    if is_available {
        return state
            .file
            .stream_file(config_id.into(), &filepath, &headers)
            .await;
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

///////////////////

///////////////////
// SETTINGS
///////////////////

/// Handler для настройки пути папки с instances на сервере
/// Ключ Права: settings.instance_path.set
async fn set_instance_path(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(request): Json<InstancePathRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::SETTINGS_INSTANCE_PATH_SET_KEY,
    )
    .await?;

    if is_available {
        state.settings.add_instance_path(&request.path).await?;
        return Ok(http::StatusCode::OK);
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

/// Handler для настройки пути папки с instances на сервере
/// Ключ Права: settings.instance_path.remove
async fn remove_instance_path(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::SETTINGS_INSTANCE_PATH_REMOVE_KEY,
    )
    .await?;

    if is_available {
        state.settings.remove_instance_path().await?;
        return Ok(http::StatusCode::OK);
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

///////////////////

///////////////////
// ROLES
///////////////////
/// Handler для создания новой роли
/// Ключ Права: role.method.create
async fn create_role(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewRoleRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(state.clone(), user_id, RightKey::CREATE_ROLE_KEY).await?;

    // action
    if is_available {
        let mut role = state.role.write().await;
        let rid = role.create_role(&payload.name).await?;
        let (k, v): (Vec<_>, Vec<_>) = rid.default_rights_setup().into_iter().unzip();

        role.add_rights(&k, &v).await?;
        return Ok(http::StatusCode::CREATED);
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

async fn update_role(
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<usize>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateRoleRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::update_role(role_id.to_string().as_str()),
    )
    .await?;

    // action
    if is_available {
        let mut role = state.role.write().await;
        let update: UpdateRole = payload.into();
        role.update_role(
            RoleId::new(role_id),
            &update.allow_keys,
            &update.remove_keys,
        )
        .await?;
        return Ok(http::StatusCode::OK);
    }

    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

async fn get_role(
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<usize>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetRoleResponse>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::get_role(role_id.to_string().as_str()),
    )
    .await?;
    // action
    if is_available {
        let role = state.role.read().await;
        return Ok(Json(role.get_role_info(RoleId::new(role_id))?));
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}

async fn list_roles(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<GetRoleResponse>>> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(state.clone(), user_id, RightKey::LIST_ROLES_KEY).await?;
    // action
    if is_available {
        let role = state.role.read().await;
        return Ok(Json(role.list_role()?));
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
}
async fn delete_role(
    State(state): State<Arc<AppState>>,
    Path(role_id): Path<usize>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    // access checking
    let is_available = check_available(
        state.clone(),
        user_id,
        RightKey::delete_role(role_id.to_string().as_str()),
    )
    .await?;

    if is_available {
        // action
        let mut role = state.role.write().await;
        let rid = RoleId::new(role_id);
        role.remove_role(rid)?;
        let (k, _): (Vec<_>, Vec<_>) = rid.default_rights_setup().into_iter().unzip();

        role.remove_rights(&k).await?;
        return Ok(http::StatusCode::OK);
    }
    Err(AppError::Forbidden(format!(
        "{} has not the needed right",
        user_id
    )))
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

async fn check_available(
    state: Arc<AppState>,
    user_id: UserId,
    key: RightKey<'static>,
) -> AppResult<bool> {
    let users_roles = state.user_profile.users_roles(user_id).await?;
    let mut set = JoinSet::new();

    for rid in users_roles {
        // Клонируем Arc<AppState>, ключ и ID роли для ПЕРЕДАЧИ ВНУТРЬ таски
        let state = Arc::clone(&state);
        let key = key.clone();

        set.spawn(async move {
            // 2. Блокировку (read) берем ИНСАЙД асинхронной таски!
            // Теперь guard ссылается на клонированный Arc, который живёт внутри таски.
            let role = state.role.read().await;
            role.is_available_action(&rid, &key).await
        });
    }

    let mut is_available = false;

    // 3. Собираем результаты
    while let Some(res) = set.join_next().await {
        // Если метод is_available_action возвращает Result<bool, Error>
        if let Ok(Ok(true)) = res {
            is_available = true;
            set.abort_all(); // Отменяем оставшиеся таски
            break;
        }

        // Если метод is_available_action возвращает просто bool (без Result):
        // if let Ok(true) = res {
        //     is_available = true;
        //     set.abort_all();
        //     break;
        // }
    }

    Ok(is_available)
}
