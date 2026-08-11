use crate::error::{PermissionError, PermissionResult};
use async_trait::async_trait;
use rumary_dto::domain::perms::PermissionNode;
use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{ContextSet, GroupSnapshot};
use std::collections::HashMap;

/// Чтение прав. Реализуйте этот трейт поверх своей БД (Postgres и т.д.).
///
/// Обязанность реализации: вернуть уже собранный список действующих нод
/// пользователя — прямые ноды + ноды всех групп с учётом наследования и
/// weight (приоритет должен быть выставлен в `source_priority`).
/// Резолвер (`resolver::resolve`) сам разберётся с wildcard и контекстом.
#[async_trait]
pub trait PermissionStore: Send + Sync + 'static {
    async fn load_effective_nodes(
        &self,
        user_id: UserId,
        ctx: &ContextSet,
    ) -> PermissionResult<Vec<PermissionNode>>;

    async fn effective_group_names(&self, user_id: UserId) -> PermissionResult<Vec<GroupName>>;

    async fn max_group_weight(&self, user_id: UserId) -> PermissionResult<GroupWeight>;

    /// Группы и вес одним вызовом. ACL-проверка нуждается в обоих значениях
    /// сразу, а по отдельности это два похода в БД с одним и тем же
    /// рекурсивным CTE — реализация поверх Postgres может схлопнуть их в один
    /// запрос. Дефолт оставлен для простых хранилищ.
    async fn group_snapshot(&self, user_id: UserId) -> PermissionResult<GroupSnapshot> {
        let groups = self.effective_group_names(user_id).await?;
        let weight = self.max_group_weight(user_id).await?;
        Ok(GroupSnapshot::new(groups, weight))
    }
}

/// Простое in-memory хранилище — для тестов, локальной разработки или
/// маленьких сервисов без БД. Наследование групп раскладывается заранее
/// при заполнении: сюда кладут уже готовый эффективный набор.
#[derive(Default, Clone)]
pub struct InMemoryStore {
    nodes: HashMap<UserId, Vec<PermissionNode>>,
    groups: HashMap<UserId, Vec<(GroupName, GroupWeight)>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_nodes(&mut self, user_id: UserId, nodes: Vec<PermissionNode>) -> &mut Self {
        self.nodes.insert(user_id, nodes);
        self
    }

    /// Группы пользователя вместе с их весами — из них считаются
    /// `effective_group_names` и `max_role_weight`.
    pub fn set_groups(
        &mut self,
        user_id: UserId,
        groups: Vec<(GroupName, GroupWeight)>,
    ) -> &mut Self {
        self.groups.insert(user_id, groups);
        self
    }
}

#[async_trait]
impl PermissionStore for InMemoryStore {
    async fn load_effective_nodes(
        &self,
        user_id: UserId,
        _ctx: &ContextSet,
    ) -> PermissionResult<Vec<PermissionNode>> {
        Ok(self.nodes.get(&user_id).cloned().unwrap_or_default())
    }

    async fn effective_group_names(&self, user_id: UserId) -> PermissionResult<Vec<GroupName>> {
        Ok(self
            .groups
            .get(&user_id)
            .map(|groups| groups.iter().map(|(name, _)| name.clone()).collect())
            .unwrap_or_default())
    }

    async fn max_group_weight(&self, user_id: UserId) -> PermissionResult<GroupWeight> {
        Ok(self
            .groups
            .get(&user_id)
            .and_then(|groups| groups.iter().map(|(_, weight)| *weight).max())
            .unwrap_or(GroupWeight::NONE))
    }
}

/// Хранилище, которое всегда отвечает ошибкой — удобно проверять, что
/// вызывающий код действительно fail-closed, а не молча пропускает запрос.
#[derive(Default, Clone, Copy)]
pub struct FailingStore;

#[async_trait]
impl PermissionStore for FailingStore {
    async fn load_effective_nodes(
        &self,
        _user_id: UserId,
        _ctx: &ContextSet,
    ) -> PermissionResult<Vec<PermissionNode>> {
        Err(PermissionError::StoreError(sqlx::Error::InvalidArgument("store is unavailable".to_owned())))
    }

    async fn effective_group_names(&self, _user_id: UserId) -> PermissionResult<Vec<GroupName>> {
        Err(PermissionError::StoreError(sqlx::Error::InvalidArgument("store is unavailable".to_owned())))
    }

    async fn max_group_weight(&self, _user_id: UserId) -> PermissionResult<GroupWeight> {
        Err(PermissionError::StoreError(sqlx::Error::InvalidArgument("store is unavailable".to_owned())))
    }
}
