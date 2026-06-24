use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use chrono::Utc;
use semver::Version;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{
        Client, LauncherBuild, LauncherUpdate, PathRuleSet, Profile, Session, SkinServiceConfig,
        User,
    },
    state::AppState,
};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/users", get(list_users))
        .route("/api/users/{user_id}/ban", post(ban_user))
        .route("/api/users/{user_id}/unban", post(unban_user))

        .route("/api/clients", post(create_client).get(list_clients))
        .route("/api/profiles", post(create_profile))
        .route("/api/profiles/{profile_id}", get(get_profile))
        .route(
            "/api/profiles/{profile_id}/validate",
            post(validate_profile),
        )
        .route("/api/installations", post(create_installation))
        .route("/api/launcher/releases", post(publish_launcher_build))
        .route(
            "/api/launcher/download/latest",
            get(get_latest_launcher_build),
        )
        .route("/api/launcher/updates/check", get(check_launcher_update))
        .route("/api/integrations/authlib", get(get_authlib_config))
        .route(
            "/api/integrations/skins",
            get(get_skin_config).put(set_skin_config),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = state
        .auth_provider
        .register(
            state.repository.clone(),
            &payload.username,
            &payload.password,
        )
        .await?;
    Ok(Json(user.into()))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<Session>> {
    let session = state
        .auth_provider
        .login(
            state.repository.clone(),
            &payload.username,
            &payload.password,
        )
        .await?;
    Ok(Json(session))
}

async fn list_users(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<UserResponse>>> {
    let users = state.list_users().await?;
    Ok(Json(users.into_iter().map(Into::into).collect()))
}

async fn ban_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(state.set_user_ban(user_id, true).await?.into()))
}

async fn unban_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(state.set_user_ban(user_id, false).await?.into()))
}

async fn create_client(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateClientRequest>,
) -> AppResult<Json<Client>> {
    Ok(Json(
        state
            .create_client(Client {
                id: Uuid::nil(),
                slug: payload.slug,
                display_name: payload.display_name,
                minecraft_version: payload.minecraft_version,
                authlib_injector_url: payload.authlib_injector_url,
                files: payload.files,
                rules: payload.rules,
                launch_arguments: payload.launch_arguments,
                created_at: Utc::now(),
            })
            .await?,
    ))
}

async fn list_clients(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Client>>> {
    Ok(Json(state.list_clients().await?))
}

async fn create_profile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProfileRequest>,
) -> AppResult<Json<Profile>> {
    Ok(Json(
        state
            .create_profile(Profile {
                id: Uuid::nil(),
                client_id: payload.client_id,
                slug: payload.slug,
                display_name: payload.display_name,
                mods: payload.mods,
                rules: payload.rules,
                created_at: Utc::now(),
            })
            .await?,
    ))
}

async fn get_profile(
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<Uuid>,
) -> AppResult<Json<Profile>> {
    Ok(Json(state.get_profile(profile_id).await?))
}

async fn validate_profile(
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<Uuid>,
) -> AppResult<Json<crate::models::ValidationReport>> {
    Ok(Json(state.validate_profile(profile_id).await?))
}

async fn create_installation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInstallationRequest>,
) -> AppResult<Json<crate::models::InstallationPlan>> {
    Ok(Json(
        state
            .create_installation(
                payload.user_id,
                payload.client_id,
                payload.profile_id,
                payload.platform,
                payload.launcher_version,
            )
            .await?,
    ))
}

async fn publish_launcher_build(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PublishLauncherBuildRequest>,
) -> AppResult<Json<LauncherBuild>> {
    Ok(Json(
        state
            .publish_launcher_build(LauncherBuild {
                id: Uuid::nil(),
                version: payload.version,
                channel: payload.channel,
                download_url: payload.download_url,
                checksum: payload.checksum,
                changelog: payload.changelog,
                published_at: Utc::now(),
            })
            .await?,
    ))
}

async fn get_latest_launcher_build(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ChannelQuery>,
) -> AppResult<Json<Option<LauncherBuild>>> {
    Ok(Json(state.get_latest_launcher_build(&query.channel).await?))
}

async fn check_launcher_update(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UpdateQuery>,
) -> AppResult<Json<LauncherUpdate>> {
    Ok(Json(
        state
            .check_launcher_update(&query.channel, &query.current_version)
            .await?,
    ))
}

async fn get_authlib_config(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<crate::models::AuthlibConfig>> {
    Ok(Json(state.minecraft_provider.authlib_config().await?))
}

async fn get_skin_config(State(state): State<Arc<AppState>>) -> AppResult<Json<SkinServiceConfig>> {
    Ok(Json(state.skin_service.get_config().await?))
}

async fn set_skin_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetSkinConfigRequest>,
) -> AppResult<Json<SkinServiceConfig>> {
    Ok(Json(
        state.skin_service.set_base_url(payload.base_url).await?,
    ))
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: Uuid,
    username: String,
    banned: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            banned: value.banned,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateClientRequest {
    slug: String,
    display_name: String,
    minecraft_version: String,
    authlib_injector_url: Option<String>,
    files: Vec<crate::models::ManagedFile>,
    rules: PathRuleSet,
    launch_arguments: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateProfileRequest {
    client_id: Uuid,
    slug: String,
    display_name: String,
    mods: Vec<crate::models::ManagedFile>,
    rules: PathRuleSet,
}

#[derive(Debug, Deserialize)]
struct CreateInstallationRequest {
    user_id: Uuid,
    client_id: Uuid,
    profile_id: Option<Uuid>,
    platform: String,
    launcher_version: Option<Version>,
}

#[derive(Debug, Deserialize)]
struct PublishLauncherBuildRequest {
    version: Version,
    channel: String,
    download_url: String,
    checksum: Option<String>,
    changelog: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChannelQuery {
    channel: String,
}

#[derive(Debug, Deserialize)]
struct UpdateQuery {
    channel: String,
    current_version: Version,
}

#[derive(Debug, Deserialize)]
struct SetSkinConfigRequest {
    base_url: Option<String>,
}
