use moka::future::Cache;
use rumary_dto::domain::perms::{GroupListQuery, GroupSummary, PermissionNode};
use rumary_dto::domain::perms::value_object::group::GroupName;
use rumary_dto::domain::perms::value_object::user::UserId;
use std::sync::Arc;
use std::time::Duration;
use rumary_perms::GroupDirectory;
use crate::error::AppResult;

const DEFAULT_TTL: Duration = Duration::from_secs(60);
const DEFAULT_CAPACITY: u64 = 10_000;

/// Всё, что обычно нужно разом для отображения одной роли — собирается
/// одним `try_join!` и кэшируется целиком, чтобы инвалидация группы была
/// одной операцией, а не четырьмя (summary/permissions/members/parents).
#[derive(Debug, Clone)]
pub struct GroupDetails {
    pub summary: GroupSummary,
    pub permissions: Vec<PermissionNode>,
    pub members: Vec<UserId>,
    pub parents: Vec<GroupName>,
}

/// Read-обёртка над `GroupDirectory` с кэшем. Список групп меняется редко
/// (создание/удаление роли — не hot path), поэтому TTL можно держать
/// заметно дольше, чем у `PermissionService`, но не бессрочно — на случай,
/// если кто-то забудет вызвать инвалидацию.
pub struct GroupsReadFacade {
    directory: Arc<dyn GroupDirectory>,
    details_cache: Cache<GroupName, Arc<GroupDetails>>,
    /// Полный список групп кэшируется только для запроса "без пагинации" —
    /// частный случай на один элемент, все листинги через limit/offset
    /// (в админке — если понадобится) идут в БД напрямую, без кэша.
    full_list_cache: Cache<(), Arc<Vec<GroupSummary>>>,
}

impl GroupsReadFacade {
    pub fn new(directory: Arc<dyn GroupDirectory>) -> Self {
        Self::with_ttl(directory, DEFAULT_TTL)
    }

    pub fn with_ttl(directory: Arc<dyn GroupDirectory>, ttl: Duration) -> Self {
        Self {
            directory,
            details_cache: Cache::builder()
                .max_capacity(DEFAULT_CAPACITY)
                .time_to_live(ttl)
                .build(),
            full_list_cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(ttl)
                .build(),
        }
    }

    /// Список групп. Кэшируется только когда `query` — дефолт (без лимита);
    /// постраничные запросы всегда идут в БД, чтобы не городить кэш-ключ
    /// под произвольные (limit, offset) комбинации ради редкого сценария.
    pub async fn list_groups(&self, query: GroupListQuery) -> AppResult<Vec<GroupSummary>> {
        if query.limit.is_some() || query.offset != 0 {
            return Ok(self.directory.list_groups(query).await?);
        }

        if let Some(cached) = self.full_list_cache.get(&()).await {
            return Ok((*cached).clone());
        }

        let fresh = self.directory.list_groups(query).await?;
        self.full_list_cache.insert((), Arc::new(fresh.clone())).await;
        Ok(fresh)
    }

    /// Полный набор данных по одной группе — то, что нужно `get_role`.
    /// Единственная точка входа для чтения деталей группы: не вызывайте
    /// `directory` напрямую по кускам, иначе кэш не защитит от лишних
    /// походов в БД.
    pub async fn get_group_details(&self, name: &GroupName) -> AppResult<Option<GroupDetails>> {
        if let Some(cached) = self.details_cache.get(name).await {
            return Ok(Some((*cached).clone()));
        }

        let Some(summary) = self.directory.get_group(name).await? else {
            return Ok(None);
        };

        let (permissions, members, parents) = tokio::try_join!(
            self.directory.list_group_permissions(name),
            self.directory.list_group_members(name),
            self.directory.list_group_parents(name),
        )?;

        let details = Arc::new(GroupDetails { summary, permissions, members, parents });
        self.details_cache.insert(name.clone(), details.clone()).await;

        Ok(Some((*details).clone()))
    }

    /// Сбросить кэш одной группы — вызывать после ЛЮБОГО изменения этой
    /// группы: вес, права, состав участников, наследование.
    pub async fn invalidate_group(&self, name: &GroupName) {
        self.details_cache.invalidate(name).await;
        // список групп мог измениться в весе/названии — проще сбросить целиком,
        // чем разбираться, повлияло ли конкретное изменение на сортировку
        self.full_list_cache.invalidate(&()).await;
    }

    /// Сбросить всё — нужно после create_group/delete_group (список групп
    /// точно изменился) и как fallback при массовых операциях.
    pub async fn invalidate_all(&self) {
        self.details_cache.invalidate_all();
        self.full_list_cache.invalidate_all();
    }
}