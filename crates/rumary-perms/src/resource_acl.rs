use crate::error::PermissionResult;
use crate::service::PermissionService;
use crate::store::PermissionStore;
use async_trait::async_trait;
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{AccessGrant, AccessMode, ResourceRef, GroupSnapshot};

#[async_trait]
pub trait ResourceAclStore: Send + Sync + 'static {
    async fn grant(
        &self,
        resource: &ResourceRef,
        grant: &AccessGrant,
        mode: AccessMode,
    ) -> PermissionResult<()>;
    async fn deny_user(
        &self,
        store: &dyn PermissionStore,
        resource: &ResourceRef,
        actor_id: UserId,
        target_user_id: UserId,
    ) -> PermissionResult<()>;
    async fn revoke(&self, resource: &ResourceRef, grant: &AccessGrant) -> PermissionResult<()>;

    async fn revoke_all_for_resource(&self, resource: &ResourceRef) -> PermissionResult<()>;

    async fn is_allowed(
        &self,
        resource: &ResourceRef,
        user_id: UserId,
        roles: &GroupSnapshot,
    ) -> PermissionResult<bool>;

    async fn is_allowed_with_bypass(
        &self,
        perms: &PermissionService,
        resource: &ResourceRef,
        user_id: UserId,
        roles: &GroupSnapshot,
    ) -> PermissionResult<bool>;
}