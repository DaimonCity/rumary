use crate::error::PermissionResult;
use async_trait::async_trait;
use rumary_dto::domain::perms::value_object::expiration::NodeExpiry;
use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{ContextSet, NodeValue};

/// Операции изменения прав/групп — то, что вызывает админка или REST-эндпоинты
/// управления правами. Отдельно от `PermissionStore` (которое только читает),
/// потому что в рантайме сервиса обычно нужно только чтение + кэш,
/// а запись используется в узком месте (админ-панель / management API).
///
/// ВАЖНО: любая запись здесь делает закэшированные наборы нод устаревшими.
/// Вызывайте `PermissionService::invalidate_user` после user-операций и
/// `invalidate_all` после group-операций — иначе изменение вступит в силу
/// только по истечении TTL кэша.
#[async_trait]
pub trait PermissionAdmin: Send + Sync + 'static {
    /// Создать новую группу. `weight` — приоритет при конфликте с другими
    /// группами пользователя (больше = важнее) и одновременно ранг группы.
    async fn create_group(&self, name: &GroupName, weight: GroupWeight) -> PermissionResult<()>;

    async fn update_group_weight(&self, name: &GroupName, weight: GroupWeight) -> PermissionResult<()>;

    /// Удалить группу целиком вместе с её правами, наследованием и
    /// членством пользователей.
    async fn delete_group(&self, name: &GroupName) -> PermissionResult<()>;

    /// Сделать группу `group` наследующей права группы `parent`.
    async fn add_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
        context: &ContextSet,
    ) -> PermissionResult<()>;

    /// Убрать наследование.
    async fn remove_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
    ) -> PermissionResult<()>;

    /// Выдать (или запретить, если `NodeValue::Deny`) право группе.
    /// Повторный вызов с тем же ключом и контекстом перезаписывает значение,
    /// а не плодит дубли нод.
    async fn set_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
        value: NodeValue,
        context: &ContextSet,
    ) -> PermissionResult<()>;

    /// Отозвать конкретное право у конкретной группы.
    async fn revoke_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
    ) -> PermissionResult<()>;


    /// Выдать (или запретить) право напрямую пользователю — в обход групп.
    /// `expires_at` = None означает бессрочное право.
    async fn set_user_permission(
        &self,
        user_id: UserId,
        key: &PermissionKey,
        value: NodeValue,
        context: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> PermissionResult<()>;

    /// Отозвать конкретное прямое право у пользователя.
    async fn revoke_user_permission(
        &self,
        user_id: UserId,
        key: &PermissionKey,
    ) -> PermissionResult<()>;

    /// Добавить пользователя в группу. `expires_at` = None — бессрочно.
    async fn add_user_to_group(
        &self,
        user_id: UserId,
        group: &GroupName,
        context: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> PermissionResult<()>;

    /// Убрать пользователя из группы.
    async fn remove_user_from_group(
        &self,
        user_id: UserId,
        group: &GroupName,
    ) -> PermissionResult<()>;
}
