use crate::PermissionResult;
use async_trait::async_trait;
use rumary_dto::domain::perms::value_object::group::GroupName;
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{GroupListQuery, GroupSummary, PermissionNode};

/// Чтение каталога групп для админки: список групп, права конкретной группы,
/// её участники, её родители по наследованию.
///
/// Отдельно от `PermissionStore` (тот читает эффективные права ОДНОГО
/// пользователя) и от `PermissionAdmin` (тот только пишет). Этот трейт нужен
/// исключительно для UI управления группами — рантайм-проверка прав
/// (`PermissionService`) им не пользуется.
#[async_trait]
pub trait GroupDirectory: Send + Sync + 'static {
    /// Все группы в системе — для списка групп.
    async fn list_groups(&self, query: GroupListQuery) -> PermissionResult<Vec<GroupSummary>>;

    /// Одна группа по имени — None, если не существует.
    async fn get_group(&self, name: &GroupName) -> PermissionResult<Option<GroupSummary>>;

    /// Собственные ноды группы (то, что выдано ЕЙ явно — без резолвинга
    /// через наследование, это отдельная забота resolver-а).
    async fn list_group_permissions(&self, name: &GroupName) -> PermissionResult<Vec<PermissionNode>>;

    /// Пользователи, состоящие в группе напрямую (без учёта наследования).
    async fn list_group_members(&self, name: &GroupName) -> PermissionResult<Vec<UserId>>;

    /// От каких групп эта группа наследует права.
    async fn list_group_parents(&self, name: &GroupName) -> PermissionResult<Vec<GroupName>>;
}