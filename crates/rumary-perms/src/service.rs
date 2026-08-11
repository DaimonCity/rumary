use crate::error::{PermissionError, PermissionResult};
use crate::resolver;
use crate::store::PermissionStore;
use moka::future::Cache;
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::value_object::user::UserId;
use rumary_dto::domain::perms::{ContextSet, PermissionNode, Tristate};
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(60);
const DEFAULT_CACHE_CAPACITY: u64 = 50_000;

/// Единая точка входа для проверки прав. Кладётся в состояние приложения
/// (`AppState`) и внедряется в любые функции/хендлеры, которым нужна проверка.
///
/// Дешёвый в клонировании: внутри Arc + Cache, клонируйте по необходимости
/// вместо того, чтобы прокидывать ссылки.
#[derive(Clone)]
pub struct PermissionService {
    store: Arc<dyn PermissionStore>,
    cache: Cache<CacheKey, Arc<Vec<PermissionNode>>>,
}

/// Ключ кэша: пользователь + хэш контекста запроса. Именованный тип вместо
/// кортежа — иначе `(UserId, u64)` ничего не говорит о том, что за u64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    user_id: UserId,
    context_hash: u64,
}

impl PermissionService {
    pub fn new(store: impl PermissionStore) -> Self {
        Self::with_cache_ttl(store, DEFAULT_CACHE_TTL)
    }

    pub fn with_cache_ttl(store: impl PermissionStore, ttl: Duration) -> Self {
        Self::from_arc(Arc::new(store), ttl)
    }

    /// Для случая, когда хранилище уже за `Arc` — например, тот же
    /// `PostgresRepo`, который лежит в `AppState` как репозиторий.
    pub fn from_arc(store: Arc<dyn PermissionStore>, ttl: Duration) -> Self {
        Self {
            store,
            cache: Cache::builder()
                .max_capacity(DEFAULT_CACHE_CAPACITY)
                .time_to_live(ttl)
                .build(),
        }
    }

    /// Доступ к хранилищу — чтобы вызывающий код не держал вторую копию
    /// того же `Arc` только ради `effective_group_names`/`role_snapshot`.
    pub fn store(&self) -> &Arc<dyn PermissionStore> {
        &self.store
    }

    /// true/false с политикой "нет ноды -> запрещено" (обычно то, что нужно).
    ///
    /// Ошибка хранилища тоже даёт false: fail-closed. Если нужно отличать
    /// "запрещено" от "не смогли проверить" — берите `try_resolve`.
    pub async fn check(&self, user_id: UserId, permission: &PermissionKey, ctx: &ContextSet) -> bool {
        self.try_resolve(user_id, permission, ctx)
            .await
            .unwrap_or(Tristate::Deny)
            .as_bool(false)
    }

    /// То же самое, но с явным дефолтом на случай Undefined (например, часть
    /// прав можно оставить "разрешено по умолчанию"). Ошибка хранилища
    /// по-прежнему трактуется как запрет, а не как Undefined.
    pub async fn check_with_default(
        &self,
        user_id: UserId,
        permission: &PermissionKey,
        ctx: &ContextSet,
        default_if_undefined: bool,
    ) -> bool {
        match self.try_resolve(user_id, permission, ctx).await {
            Ok(state) => state.as_bool(default_if_undefined),
            Err(_) => false,
        }
    }

    /// Возвращает Ok(()) или Err(PermissionError) — удобно использовать
    /// с `?` прямо внутри бизнес-логики. В отличие от `check`, недоступность
    /// хранилища здесь не превращается в "просто запрещено": наверх уедет
    /// `StoreError`, и хендлер ответит 500, а не 403.
    pub async fn require(
        &self,
        user_id: UserId,
        permission: &PermissionKey,
        ctx: &ContextSet,
    ) -> PermissionResult<()> {
        if self.try_resolve(user_id, permission, ctx).await?.as_bool(false) {
            Ok(())
        } else {
            Err(PermissionError::Denied(permission.clone()))
        }
    }

    /// Точный Tristate, если вызывающему коду важно отличать явный запрет
    /// от отсутствия права.
    pub async fn try_resolve(
        &self,
        user_id: UserId,
        permission: &PermissionKey,
        ctx: &ContextSet,
    ) -> PermissionResult<Tristate> {
        let nodes = self.effective_nodes(user_id, ctx).await?;
        Ok(resolver::resolve(permission, ctx, &nodes))
    }

    /// Сбросить кэш конкретного пользователя — вызывать после изменения его
    /// прямых прав или членства в группах.
    ///
    /// Инвалидация точечная (по префиксу ключа), поэтому изменение прав одного
    /// пользователя не выбрасывает кэш всех остальных, как было раньше.
    pub async fn invalidate_user(&self, user_id: UserId) {
        self.cache
            .invalidate_entries_if(move |key, _| key.user_id == user_id)
            .ok();
    }

    /// Сбросить кэш целиком — нужно после изменения прав ГРУППЫ: заранее
    /// неизвестно, кто в неё входит напрямую или через наследование.
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    async fn effective_nodes(
        &self,
        user_id: UserId,
        ctx: &ContextSet,
    ) -> PermissionResult<Arc<Vec<PermissionNode>>> {
        let cache_key = CacheKey {
            user_id,
            context_hash: ctx.cache_key(),
        };

        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        // Ошибку загрузки НЕ кэшируем и не превращаем в пустой набор:
        // пустой набор означал бы "прав нет" и залёг бы в кэш на весь TTL.
        let fresh = Arc::new(self.store.load_effective_nodes(user_id, ctx).await?);
        self.cache.insert(cache_key, fresh.clone()).await;

        Ok(fresh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{FailingStore, InMemoryStore};
    use rumary_dto::domain::perms::NodeValue;
    use rumary_dto::domain::perms::value_object::node::SourcePriority;
    use uuid::Uuid;

    fn key(value: &str) -> PermissionKey {
        PermissionKey::try_from(value).expect("valid permission key")
    }

    #[tokio::test]
    async fn check_resolves_granted_permission() {
        let user_id = UserId::from(Uuid::new_v4());
        let mut store = InMemoryStore::new();
        store.set_nodes(
            user_id,
            vec![PermissionNode::permanent(
                key("configuration.*"),
                NodeValue::Allow,
                SourcePriority::USER,
            )],
        );

        let perms = PermissionService::new(store);
        assert!(perms.check(user_id, &key("configuration.get"), &ContextSet::empty()).await);
        assert!(!perms.check(user_id, &key("instance.get"), &ContextSet::empty()).await);
    }

    #[tokio::test]
    async fn check_is_fail_closed_on_store_error() {
        let perms = PermissionService::new(FailingStore);
        let user_id = UserId::from(Uuid::new_v4());

        assert!(!perms.check(user_id, &key("configuration.get"), &ContextSet::empty()).await);
        assert!(
            !perms
                .check_with_default(user_id, &key("configuration.get"), &ContextSet::empty(), true)
                .await
        );
    }

    #[tokio::test]
    async fn require_propagates_store_error_instead_of_denying() {
        let perms = PermissionService::new(FailingStore);
        let result = perms
            .require(UserId::from(Uuid::new_v4()), &key("configuration.get"), &ContextSet::empty())
            .await;

        assert!(matches!(result, Err(PermissionError::StoreError(_))));
    }

    #[tokio::test]
    async fn store_error_is_not_cached_as_empty_node_set() {
        let user_id = UserId::from(Uuid::new_v4());
        let perms = PermissionService::new(FailingStore);

        let _ = perms.check(user_id, &key("configuration.get"), &ContextSet::empty()).await;
        // второй вызов снова обращается к хранилищу, а не отдаёт закэшированный
        // пустой набор — иначе временный сбой БД "запрещал" бы всё на весь TTL
        assert!(matches!(
            perms
                .require(user_id, &key("configuration.get"), &ContextSet::empty())
                .await,
            Err(PermissionError::StoreError(_))
        ));
    }
}
