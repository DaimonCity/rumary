use crate::error::{AppError, AppResult};
use crate::service::auth::AuthenticatedUser;
use crate::service::permissions::ResourceAction;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, patch, put};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use rumary_dto::domain::api::LoginOutcome;
use rumary_dto::domain::api::group::ListGroupsQuery;
use rumary_dto::domain::api::share_target::ShareTarget;
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::domain::api::value_object::instance::InstanceId;
use rumary_dto::domain::api::value_object::user::UserId as ApiUserId;
use rumary_dto::domain::perms::value_object::expiration::NodeExpiry;
use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::value_object::resource::ResourceType;
use rumary_dto::domain::perms::value_object::user::UserId as PermsUserId;
use rumary_dto::domain::perms::{ContextSet, GroupListQuery, NodeValue};
use rumary_dto::dto::api::request::NewGroupRequest;
use rumary_dto::dto::api::request::share_target::ShareTargetRequest;
use rumary_dto::dto::api::request::{
    AddGroupMemberRequest, AddGroupParentRequest, CreateUserBanRequest, DeleteMeRequest,
    InstancePathRequest, LoginRequest, NewConfigurationRequest, NewInstanceRequest,
    RegisterRequest, RevokeUserBanRequest, TotpLoginRequest, UpdateConfigurationRequest,
    UpdateGroupPermissionsRequest, UpdateGroupWeightRequest, UpdateInstanceRequest,
};
use rumary_dto::dto::api::response::group::{
    GetGroupResponse, GroupPermissionResponse, GroupSummaryResponse,
};
use rumary_dto::dto::api::response::{CapabilitiesResponse, ProfileResponse};
use rumary_dto::dto::api::response::{
    GetConfigurationResponse, GetInstanceResponse, SessionTokensResponse, TokenResponse,
    UserBanResponse,
};
use rumary_perms::{PermissionError, require_outranks};
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
        .route("/api/v1/users/me", get(user_get_me).delete(delete_me))
        .route("/api/v1/users/me/capabilities", get(user_capabilities))
        .route("/api/v1/user/{user_id}", get(user_get))
        .route(
            "/api/v1/user/{user_id}/bans",
            get(list_user_bans).post(create_user_ban),
        )
        .route(
            "/api/v1/user/{user_id}/bans/{ban_id}",
            delete(revoke_user_ban),
        )
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
        .route(
            "/api/v1/instance/{instance_id}/configurations",
            get(list_configuration),
        )
        .route("/api/v1/groups", get(list_groups).post(create_group))
        .route("/api/v1/groups/{name}", get(get_group).delete(delete_group))
        .route("/api/v1/groups/{name}/weight", put(update_group_weight))
        .route(
            "/api/v1/groups/{name}/permissions",
            patch(update_group_permissions),
        )
        .route("/api/v1/groups/{name}/parents", post(add_group_parent))
        .route(
            "/api/v1/groups/{name}/parents/{parent}",
            delete(remove_group_parent),
        )
        .route("/api/v1/groups/{name}/members", post(add_group_member))
        .route(
            "/api/v1/groups/{name}/members/{user_id}",
            delete(remove_group_member),
        )
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

///////////////////
// AUTH
///////////////////

async fn register(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(CookieJar, Json<TokenResponse>)> {
    let tokens = state.auth.register(payload).await?;
    let user = {
        let user = state
            .auth
            .authenticate_access_token(&tokens.access_token)
            .await?;
        state.user_profile.get(user.id).await?
    };

    let resource = user.resource_ref()?;
    state
        .perms_admin
        .add_user_to_group(
            user.id.into(),
            &GroupName::try_from("user".to_string())?,
            &ContextSet::empty(),
            None,
        )
        .await?;
    state
        .perms
        .register_created_resource(user.id.into(), &resource, &[])
        .await?;
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

/// Ключ Права: auth.session.update
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

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("auth.session")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    let tokens = state.auth.refresh(&refresh_token, user_session).await?;
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
    state.auth.logout(auth_user.id).await?;
    Ok(clear_session_cookies(jar, &state))
}

///////////////////
// USERS
///////////////////

/// Ключ Права: user.get (RBAC) + ACL по конкретному профилю
async fn user_get_me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<ProfileResponse>> {
    user_get(Path(*auth_user.id), State(state), auth_user).await
}

const UI_PERMISSION_KEYS: &[&str] = &[
    "*",
    "instance.list",
    "instance.create",
    "instance.update",
    "instance.delete",
    "instance.configurations.list",
    "configuration.get",
    "configuration.create",
    "configuration.update",
    "configuration.delete",
    "configuration.download",
    "user.ban",
    "user.ban.permanent",
    "user.unban",
    "settings.instance_path.update",
    "settings.instance_path.delete",
    "group.list",
    "group.get",
    "group.create",
    "group.delete",
    "group.weight.update",
    "group.permissions.update",
    "group.parents.update",
    "group.members.create",
    "group.members.delete",
];

async fn user_capabilities(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<CapabilitiesResponse>> {
    let user_id = auth_user.id.into();
    let context = ContextSet::empty();
    let mut permissions = Vec::new();

    for raw_key in UI_PERMISSION_KEYS {
        let key = PermissionKey::try_from(*raw_key).map_err(PermissionError::from)?;
        if state.perms.service().check(user_id, &key, &context).await {
            permissions.push((*raw_key).to_string());
        }
    }

    Ok(Json(CapabilitiesResponse { permissions }))
}

/// Ключ Права: user.get (RBAC) + ACL по конкретному профилю
async fn user_get(
    Path(user_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<ProfileResponse>> {
    let actor_id = auth_user.id.into();
    let user = state.user_profile.get(user_id.into()).await?;
    let resource = user.resource_ref()?;

    state
        .perms
        .require_resource_access(
            actor_id,
            &resource,
            ResourceAction::Get,
            user.is_public,
            &ContextSet::empty(),
        )
        .await?;

    let has_totp = state.totp.is_enabled(user_id.into()).await?;
    Ok(Json(user.to_profile_response(has_totp)))
}

/// Ключ права: user.ban + строгая проверка ранга.
async fn create_user_ban(
    Path(target_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateUserBanRequest>,
) -> AppResult<(http::StatusCode, Json<UserBanResponse>)> {
    let actor_id = auth_user.id;
    let target_id = ApiUserId::from(target_id);
    state
        .perms
        .require_action_on_user(
            actor_id.into(),
            target_id.into(),
            &ResourceType::try_from("user")?,
            ResourceAction::Ban,
            &ContextSet::empty(),
        )
        .await?;

    if payload.expires_at.is_none() {
        state
            .perms
            .require_action_on_user(
                actor_id.into(),
                target_id.into(),
                &ResourceType::try_from("user")?,
                ResourceAction::BanPermanent,
                &ContextSet::empty(),
            )
            .await?;
    }

    let ban = state
        .moderation
        .ban_user(actor_id, target_id, payload)
        .await?;
    Ok((http::StatusCode::CREATED, Json(ban.into())))
}

/// Ключ права: user.ban + строгая проверка ранга.
async fn list_user_bans(
    Path(target_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<UserBanResponse>>> {
    let actor_id = auth_user.id;
    let target_id = ApiUserId::from(target_id);
    state
        .perms
        .require_action_on_user(
            actor_id.into(),
            target_id.into(),
            &ResourceType::try_from("user")?,
            ResourceAction::Ban,
            &ContextSet::empty(),
        )
        .await?;

    let bans = state.moderation.list_user_bans(target_id).await?;
    Ok(Json(bans.into_iter().map(Into::into).collect()))
}

/// Ключ права: user.unban + строгая проверка ранга.
async fn revoke_user_ban(
    Path((target_id, ban_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<RevokeUserBanRequest>,
) -> AppResult<Json<UserBanResponse>> {
    let actor_id = auth_user.id;
    let target_id = ApiUserId::from(target_id);
    state
        .perms
        .require_action_on_user(
            actor_id.into(),
            target_id.into(),
            &ResourceType::try_from("user")?,
            ResourceAction::Unban,
            &ContextSet::empty(),
        )
        .await?;

    let ban = state
        .moderation
        .revoke_user_ban(actor_id, target_id, ban_id.into(), payload)
        .await?;
    Ok(Json(ban.into()))
}

/// Ключ Права: user.delete (RBAC) + ACL по конкретному профилю
async fn delete_me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    auth_user: AuthenticatedUser,
    Json(payload): Json<DeleteMeRequest>,
) -> AppResult<CookieJar> {
    let user_id = auth_user.id;

    let user = state.user_profile.get(user_id).await?;
    let resource = user.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Delete,
            user.is_public,
            &ContextSet::empty(),
        )
        .await?;

    state
        .user_profile
        .delete(user_id, &payload.password)
        .await?;
    state.perms.cleanup_deleted_resource(&resource).await?;

    Ok(clear_session_cookies(jar, &state))
}

///////////////////
// INSTANCE
///////////////////

/// Ключ Права: instance.create
async fn create_instance(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewInstanceRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("instance")?,
            ResourceAction::Create,
            &ContextSet::empty(),
        )
        .await?;
    let share_with: Vec<ShareTarget> = payload
        .share_with
        .iter()
        .cloned()
        .map(ShareTargetRequest::into_domain)
        .collect::<Result<_, _>>()?;

    let instance = state.instance.create(payload.try_into()?).await?;

    state
        .perms
        .register_created_resource(user_id.into(), &instance.resource_ref()?, &share_with)
        .await?;

    Ok(http::StatusCode::CREATED)
}

/// Ключ Права: instance.get
async fn get_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetInstanceResponse>> {
    let user_id = auth_user.id;
    let instance_id = InstanceId::from(instance_id);

    let instance = state.instance.get(instance_id).await?;
    let resource = instance.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Get,
            instance.is_public,
            &ContextSet::empty(),
        )
        .await?;

    Ok(Json(instance.into()))
}

/// Ключ Права: instance.list
async fn list_instance(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<GetInstanceResponse>>> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("instance")?,
            ResourceAction::List,
            &ContextSet::empty(),
        )
        .await?;

    let all = state.instance.list().await?;

    let mut visible = Vec::new();
    for instance in all {
        let resource = instance.resource_ref()?;
        if state
            .perms
            .can_access_resource(
                user_id.into(),
                &resource,
                ResourceAction::Get,
                instance.is_public,
                &ContextSet::empty(),
            )
            .await
        {
            visible.push(instance.into());
        }
    }

    Ok(Json(visible))
}

/// Ключ Права: instance.update
async fn update_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateInstanceRequest>,
) -> AppResult<Json<GetInstanceResponse>> {
    let user_id = auth_user.id;
    let instance_id = InstanceId::from(instance_id);

    let instance = state.instance.get(instance_id).await?;
    let resource = instance.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Update,
            instance.is_public,
            &ContextSet::empty(),
        )
        .await?;

    let updated = state
        .instance
        .update(instance_id, payload.try_into()?)
        .await?;
    Ok(Json(updated.into()))
}

/// Ключ Права: instance.delete
async fn delete_instance(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;
    let instance_id = InstanceId::from(instance_id);

    let instance = state.instance.get(instance_id).await?;
    let resource = instance.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Delete,
            instance.is_public,
            &ContextSet::empty(),
        )
        .await?;

    state.instance.delete(instance_id).await?;
    state.perms.cleanup_deleted_resource(&resource).await?;

    Ok(http::StatusCode::NO_CONTENT)
}

///////////////////
// CONFIGURATION
///////////////////

/// Ключ Права: configuration.create
async fn create_configuration(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewConfigurationRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("configuration")?,
            ResourceAction::Create,
            &ContextSet::empty(),
        )
        .await?;

    let share_with: Vec<ShareTarget> = payload
        .share_with
        .iter()
        .cloned()
        .map(ShareTargetRequest::into_domain)
        .collect::<Result<_, _>>()?;
    let config = state.config.create(payload.try_into()?).await?;

    state
        .perms
        .register_created_resource(user_id.into(), &config.resource_ref()?, &share_with)
        .await?;

    Ok(http::StatusCode::CREATED)
}

/// Ключ Права: configuration.get
async fn get_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetConfigurationResponse>> {
    let user_id = auth_user.id;
    let config_id = ConfigurationId::from(config_id);

    let config = state.config.get(config_id).await?;
    let resource = config.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Get,
            config.is_public,
            &ContextSet::empty(),
        )
        .await?;

    Ok(Json(config.into()))
}

/// Ключ Права: instance.configurations.list
async fn list_configuration(
    Path(instance_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<Vec<GetConfigurationResponse>>> {
    let user_id = auth_user.id;
    let instance_id = InstanceId::from(instance_id);

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("instance.configurations")?,
            ResourceAction::List,
            &ContextSet::empty(),
        )
        .await?;

    let all = state.config.list_for_instance(instance_id).await?;

    let mut visible = Vec::new();

    for config in all {
        let resource = config.resource_ref()?;
        if state
            .perms
            .can_access_resource(
                user_id.into(),
                &resource,
                ResourceAction::Get,
                config.is_public,
                &ContextSet::empty(),
            )
            .await
        {
            visible.push(config.into());
        }
    }

    Ok(Json(visible))
}

/// Ключ Права: configuration.update
async fn update_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateConfigurationRequest>,
) -> AppResult<Json<GetConfigurationResponse>> {
    let user_id = auth_user.id;
    let config_id = ConfigurationId::from(config_id);

    let config = state.config.get(config_id).await?;
    let resource = config.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Update,
            config.is_public,
            &ContextSet::empty(),
        )
        .await?;

    let updated = state.config.update(config_id, payload.try_into()?).await?;
    Ok(Json(updated.into()))
}

/// Ключ Права: configuration.delete
async fn delete_configuration(
    Path(config_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;
    let config_id = ConfigurationId::from(config_id);

    let config = state.config.get(config_id).await?;
    let resource = config.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Delete,
            config.is_public,
            &ContextSet::empty(),
        )
        .await?;

    state.config.delete(config_id).await?;
    state.perms.cleanup_deleted_resource(&resource).await?;

    Ok(http::StatusCode::NO_CONTENT)
}

/// Ключ Права: configuration.download
async fn download_file_handler(
    Path((config_id, filepath)): Path<(Uuid, PathBuf)>,
    headers: http::HeaderMap,
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::Response<Body>> {
    let user_id = auth_user.id;
    let config_id = ConfigurationId::from(config_id);

    let config = state.config.get(config_id).await?;
    let resource = config.resource_ref()?;

    state
        .perms
        .require_resource_access(
            user_id.into(),
            &resource,
            ResourceAction::Download,
            config.is_public,
            &ContextSet::empty(),
        )
        .await?;

    state.file.stream_file(config_id, &filepath, &headers).await
}

///////////////////
// SETTINGS
///////////////////

/// Ключ Права: settings.instance_path.update
async fn set_instance_path(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(request): Json<InstancePathRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("settings.instance_path")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    state.settings.set_instance_path(&request.path).await?;
    Ok(http::StatusCode::OK)
}

/// Ключ Права: settings.instance_path.delete
async fn remove_instance_path(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("settings.instance_path")?,
            ResourceAction::Delete,
            &ContextSet::empty(),
        )
        .await?;

    state.settings.remove_instance_path().await?;
    Ok(http::StatusCode::OK)
}

///////////////////
// GROUPS
///////////////////

/// Ключ Права: group.create
/// Вес новой группы не может быть >= веса создателя — иначе writer с правом
/// group.create мог бы создать себе группу тяжелее admin.
async fn create_group(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<NewGroupRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group")?,
            ResourceAction::Create,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&payload.name)?;
    let weight = GroupWeight::new(payload.weight)?;

    require_actor_outweighs(&state, user_id.into(), weight, "create").await?;

    state.perms_admin.create_group(&name, weight).await?;

    Ok(http::StatusCode::CREATED)
}

/// Ключ Права: group.delete
async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group")?,
            ResourceAction::Delete,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;
    require_actor_outweighs_group(&state, user_id.into(), &name, "delete").await?;
    state.perms_admin.delete_group(&name).await?;

    Ok(http::StatusCode::NO_CONTENT)
}

/// Ключ Права: group.weight.update
async fn update_group_weight(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateGroupWeightRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.weight")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;
    let weight = GroupWeight::new(payload.weight)?;
    require_actor_outweighs(&state, user_id.into(), weight, "assign").await?;
    state.perms_admin.update_group_weight(&name, weight).await?;

    Ok(http::StatusCode::OK)
}

/// Ключ Права: group.permissions.update
async fn update_group_permissions(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateGroupPermissionsRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.permissions")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;
    require_actor_outweighs_group(&state, user_id.into(), &name, "modify permissions of").await?;

    // защита от эскалации: actor не может выдать группе право, которым
    // сам не обладает — иначе group.permissions.update само по себе
    // становится обходом всей RBAC-модели
    for grant in &payload.grant {
        if grant.allow {
            let key = parse_permission_key(&grant.key)?;
            let has_it = state
                .perms
                .service()
                .check(user_id.into(), &key, &ContextSet::empty())
                .await;
            if !has_it {
                return Err(AppError::Forbidden(format!(
                    "cannot grant permission '{}' you do not hold yourself",
                    grant.key
                )));
            }
        }
    }

    for grant in payload.grant {
        let key = parse_permission_key(&grant.key)?;
        let value = NodeValue::from(grant.allow);
        state
            .perms_admin
            .set_group_permission(&name, &key, value, &ContextSet::empty())
            .await?;
    }

    for raw_key in payload.revoke {
        let key = parse_permission_key(&raw_key)?;
        state
            .perms_admin
            .revoke_group_permission(&name, &key)
            .await?;
    }

    Ok(http::StatusCode::OK)
}

/// Ключ Права: group.parents.update
async fn add_group_parent(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<AddGroupParentRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.parents")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    let group = parse_group_name(&group_name)?;
    let parent = parse_group_name(&payload.parent)?;
    require_actor_outweighs_group(&state, user_id.into(), &group, "modify inheritance of").await?;
    require_actor_outweighs_group(&state, user_id.into(), &parent, "attach as parent").await?;
    state
        .perms_admin
        .add_group_parent(&group, &parent, &ContextSet::empty())
        .await?;

    Ok(http::StatusCode::OK)
}

/// Ключ Права: group.parents.update
async fn remove_group_parent(
    State(state): State<Arc<AppState>>,
    Path((group_name, parent_name)): Path<(String, String)>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.parents")?,
            ResourceAction::Update,
            &ContextSet::empty(),
        )
        .await?;

    let group = parse_group_name(&group_name)?;
    let parent = parse_group_name(&parent_name)?;
    require_actor_outweighs_group(&state, user_id.into(), &group, "modify inheritance of").await?;
    state
        .perms_admin
        .remove_group_parent(&group, &parent)
        .await?;

    Ok(http::StatusCode::OK)
}

/// Ключ Права: group.members.create
async fn add_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<AddGroupMemberRequest>,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.members")?,
            ResourceAction::Create,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;
    let target_user_id = payload.user_id.into();

    // ранг: actor должен быть выше самого target-пользователя
    require_outranks(
        state.perms.service().store().as_ref(),
        user_id.into(),
        target_user_id,
    )
    .await?;

    // вес: actor не может выдать группу тяжелее или равную своей — иначе
    // добавление в группу становится обходным способом самоповышения
    require_actor_outweighs_group(&state, user_id.into(), &name, "grant").await?;

    let expires_at = payload.expires_at.map(NodeExpiry::new);
    state
        .perms_admin
        .add_user_to_group(target_user_id, &name, &ContextSet::empty(), expires_at)
        .await?;

    Ok(http::StatusCode::CREATED)
}

/// Ключ Права: group.members.delete
async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    Path((group_name, target_user_id)): Path<(String, Uuid)>,
    auth_user: AuthenticatedUser,
) -> AppResult<http::StatusCode> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group.members")?,
            ResourceAction::Delete,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;
    let target_user_id = target_user_id.into();

    require_outranks(
        state.perms.service().store().as_ref(),
        user_id.into(),
        target_user_id,
    )
    .await?;
    require_actor_outweighs_group(&state, user_id.into(), &name, "revoke").await?;

    state
        .perms_admin
        .remove_user_from_group(target_user_id, &name)
        .await?;

    Ok(http::StatusCode::NO_CONTENT)
}
///////////////////

///////////////////
// GROUPS — read path через GroupDirectory
///////////////////
/// Ключ Права: group.list
async fn list_groups(
    State(state): State<Arc<AppState>>,
    auth_user: AuthenticatedUser,
    Query(query): Query<ListGroupsQuery>,
) -> AppResult<Json<Vec<GroupSummaryResponse>>> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group")?,
            ResourceAction::List,
            &ContextSet::empty(),
        )
        .await?;

    let groups = state
        .group_read
        .list_groups(GroupListQuery {
            limit: query.limit,
            offset: query.offset,
        })
        .await?;

    Ok(Json(
        groups
            .into_iter()
            .map(|g| GroupSummaryResponse {
                name: g.name.as_str().to_string(),
                weight: g.weight.get(),
            })
            .collect(),
    ))
}

/// Ключ Права: group.get
async fn get_group(
    State(state): State<Arc<AppState>>,
    Path(group_name): Path<String>,
    auth_user: AuthenticatedUser,
) -> AppResult<Json<GetGroupResponse>> {
    let user_id = auth_user.id;

    state
        .perms
        .require_action(
            user_id.into(),
            &ResourceType::try_from("group")?,
            ResourceAction::Get,
            &ContextSet::empty(),
        )
        .await?;

    let name = parse_group_name(&group_name)?;

    let details = state
        .group_read
        .get_group_details(&name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("group {group_name}")))?;

    Ok(Json(GetGroupResponse {
        name: details.summary.name.as_str().to_string(),
        weight: details.summary.weight.get(),
        permissions: details
            .permissions
            .iter()
            .map(GroupPermissionResponse::from)
            .collect(),
        members: details.members.into_iter().map(Uuid::from).collect(),
        parents: details
            .parents
            .into_iter()
            .map(|p| p.as_str().to_string())
            .collect(),
    }))
}
///////////////////

///////////////////
// UTIL FUNCTIONS FOR API
///////////////////

/// Проверяет, что вес группы `target` строго меньше веса самого тяжёлого
/// ранга actor'а — общее правило для всех операций над существующей группой
/// (delete, изменение веса/прав/наследования/участников): actor не должен
/// мочь трогать группу тяжелее или равную себе.
///
/// `action` — что именно запрещено, для текста ошибки (например
/// "delete", "modify permissions of", "modify inheritance of").
async fn require_actor_outweighs_group(
    state: &AppState,
    user_id: PermsUserId,
    target: &GroupName,
    action: &str,
) -> AppResult<()> {
    let target_weight = state
        .group_read
        .get_group_details(target)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("group {}", target.as_str())))?
        .summary
        .weight;

    let actor_weight = state
        .perms
        .service()
        .store()
        .max_group_weight(user_id)
        .await?;

    if target_weight >= actor_weight {
        return Err(AppError::Forbidden(format!(
            "cannot {action} a group with weight equal to or higher than your own"
        )));
    }

    Ok(())
}

/// То же самое, но когда сравниваемый вес уже на руках (создание группы,
/// присвоение нового веса) — без похода в group_read за существующей записью.
async fn require_actor_outweighs(
    state: &AppState,
    user_id: PermsUserId,
    weight: GroupWeight,
    action: &str,
) -> AppResult<()> {
    let actor_weight = state
        .perms
        .service()
        .store()
        .max_group_weight(user_id)
        .await?;

    if weight >= actor_weight {
        return Err(AppError::Forbidden(format!(
            "cannot {action} a group with weight equal to or higher than your own"
        )));
    }

    Ok(())
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

fn parse_group_name(raw: &str) -> AppResult<GroupName> {
    Ok(GroupName::try_from(raw)?)
}

fn parse_permission_key(raw: &str) -> AppResult<PermissionKey> {
    Ok(PermissionKey::try_from(raw).map_err(PermissionError::from)?)
}
