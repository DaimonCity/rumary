use crate::error::AppResult;
use crate::service::group_read::GroupsReadFacade;
use rumary_dto::domain::perms::value_object::expiration::NodeExpiry;
use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{ContextSet, NodeValue};
use rumary_perms::{PermissionAdmin, PermissionService};
use std::sync::Arc;
pub struct PermissionsAdminFacade {
    perms: Arc<PermissionService>,
    admin: Arc<dyn PermissionAdmin>,
    groups_read: Arc<GroupsReadFacade>, // новое поле
}

impl PermissionsAdminFacade {
    pub fn new(
        perms: PermissionService,
        admin: impl PermissionAdmin,
        groups_read: GroupsReadFacade,
    ) -> Self {
        Self {
            perms: Arc::new(perms),
            admin: Arc::new(admin),
            groups_read: Arc::new(groups_read),
        }
    }

    pub fn from_arc(
        perms: Arc<PermissionService>,
        admin: Arc<dyn PermissionAdmin>,
        groups_read: Arc<GroupsReadFacade>,
    ) -> Self {
        Self {
            perms,
            admin,
            groups_read,
        }
    }

    /// Создать роль. Инвалидация кэша здесь не нужна — новая, ещё никому
    /// неназначенная роль не может быть в чьём-то закэшированном наборе нод.
    pub async fn create_group(&self, name: &GroupName, weight: GroupWeight) -> AppResult<()> {
        self.admin.create_group(name, weight).await?;
        self.groups_read.invalidate_all().await;
        Ok(())
    }

    /// Изменить вес существующей роли. Меняет резолвинг конфликтов для всех,
    /// у кого есть эта роль — invalidate_all обязателен, не точечная инвалидация.
    pub async fn update_group_weight(&self, name: &GroupName, weight: GroupWeight) -> AppResult<()> {
        self.admin.update_group_weight(name, weight).await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_all().await;
        Ok(())
    }

    /// Удалить роль. Инвалидируем весь кэш — заранее неизвестно, у кого она
    /// была в эффективном наборе нод.
    pub async fn delete_group(&self, name: &GroupName) -> AppResult<()> {
        self.admin.delete_group(name).await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_all().await;
        Ok(())
    }

    pub async fn add_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
        ctx: &ContextSet,
    ) -> AppResult<()> {
        self.admin.add_group_parent(group, parent, ctx).await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    pub async fn remove_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
    ) -> AppResult<()> {
        self.admin.remove_group_parent(group, parent).await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    pub async fn set_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
        value: NodeValue,
        ctx: &ContextSet,
    ) -> AppResult<()> {
        self.admin
            .set_group_permission(group, key, value, ctx)
            .await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    pub async fn revoke_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
    ) -> AppResult<()> {
        self.admin.revoke_group_permission(group, key).await?;
        self.perms.invalidate_all().await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    /// Инвалидация точечная: конкретный пользователь — то единственное место,
    /// где не нужен invalidate_all, потому что изменение затрагивает только его.
    pub async fn add_user_to_group(
        &self,
        user_id: UserId,
        group: &GroupName,
        ctx: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> AppResult<()> {
        self.admin
            .add_user_to_group(user_id, group, ctx, expires_at)
            .await?;
        self.perms.invalidate_user(user_id).await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    pub async fn remove_user_from_group(
        &self,
        user_id: UserId,
        group: &GroupName,
    ) -> AppResult<()> {
        self.admin.remove_user_from_group(user_id, group).await?;
        self.perms.invalidate_user(user_id).await;
        self.groups_read.invalidate_group(group).await;
        Ok(())
    }

    pub async fn set_user_permission(
        &self,
        user_id: UserId,
        key: &PermissionKey,
        value: NodeValue,
        ctx: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> AppResult<()> {
        self.admin
            .set_user_permission(user_id, key, value, ctx, expires_at)
            .await?;
        self.perms.invalidate_user(user_id).await;
        Ok(())
    }

    pub async fn revoke_user_permission(
        &self,
        user_id: UserId,
        key: &PermissionKey,
    ) -> AppResult<()> {
        self.admin.revoke_user_permission(user_id, key).await?;
        self.perms.invalidate_user(user_id).await;
        Ok(())
    }
}
