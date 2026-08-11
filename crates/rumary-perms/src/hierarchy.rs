use crate::error::{PermissionError, PermissionResult};
use crate::store::PermissionStore;
use rumary_dto::domain::perms::value_object::user::UserId;

/// Проверка "ранга": actor может действовать на target, только если у actor
/// строго больший максимальный weight группы, чем у target.
///
/// Это ОТДЕЛЬНАЯ проверка от `PermissionService::check`/`require` — та
/// отвечает "может ли группа actor-а вообще выполнять действие такого рода",
/// эта — "выше ли actor по рангу, чем конкретная цель действия". Для
/// действий, направленных на другого пользователя (delete, ban, demote,
/// изменение чужих прав), нужны ОБЕ проверки одновременно.
///
/// Actor не выше самого себя: `actor_id == target_id` всегда даёт false, и
/// это правильно — самоповышение и самозапрет должны идти через отдельные,
/// осознанные операции, а не через общую проверку ранга.
pub async fn actor_outranks_target(
    store: &(impl PermissionStore + ?Sized),
    actor_id: UserId,
    target_id: UserId,
) -> PermissionResult<bool> {
    if actor_id == target_id {
        return Ok(false);
    }

    let actor_weight = store.max_group_weight(actor_id).await?;
    let target_weight = store.max_group_weight(target_id).await?;

    Ok(actor_weight > target_weight)
}

/// То же самое, но сразу как Result с понятной ошибкой — удобно с `?`.
pub async fn require_outranks(
    store: &(impl PermissionStore + ?Sized),
    actor_id: UserId,
    target_id: UserId,
) -> PermissionResult<()> {
    if actor_outranks_target(store, actor_id, target_id).await? {
        Ok(())
    } else {
        Err(PermissionError::InsufficientRank(
            "actor does not outrank target user",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryStore;
    use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
    use uuid::Uuid;

    fn group(name: &str, weight: i32) -> (GroupName, GroupWeight) {
        (
            GroupName::try_from(name).expect("valid group name"),
            GroupWeight::new(weight).expect("valid weight"),
        )
    }

    #[tokio::test]
    async fn higher_weight_outranks_lower() {
        let admin = UserId::from(Uuid::new_v4());
        let writer = UserId::from(Uuid::new_v4());

        let mut store = InMemoryStore::new();
        store.set_groups(admin, vec![group("admin", 30)]);
        store.set_groups(writer, vec![group("writer", 15)]);

        assert!(actor_outranks_target(&store, admin, writer).await.unwrap());
        assert!(!actor_outranks_target(&store, writer, admin).await.unwrap());
    }

    #[tokio::test]
    async fn peers_do_not_outrank_each_other() {
        let writer_1 = UserId::from(Uuid::new_v4());
        let writer_2 = UserId::from(Uuid::new_v4());

        let mut store = InMemoryStore::new();
        store.set_groups(writer_1, vec![group("writer", 15)]);
        store.set_groups(writer_2, vec![group("writer", 15)]);

        assert!(!actor_outranks_target(&store, writer_1, writer_2).await.unwrap());
        assert!(matches!(
            require_outranks(&store, writer_1, writer_2).await,
            Err(PermissionError::InsufficientRank(_))
        ));
    }

    #[tokio::test]
    async fn user_does_not_outrank_self() {
        let admin = UserId::from(Uuid::new_v4());
        let mut store = InMemoryStore::new();
        store.set_groups(admin, vec![group("admin", 30)]);

        assert!(!actor_outranks_target(&store, admin, admin).await.unwrap());
    }
}
