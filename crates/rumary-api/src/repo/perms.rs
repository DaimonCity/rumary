//! Реализация трейтов `rumary-perms` поверх `PostgresRepo`.
//!
//! Держим отдельно от `db.rs`: там репозитории предметной области, здесь —
//! подсистема прав со своим набором таблиц (`groups`, `group_inheritance`,
//! `permission_nodes`, `user_groups`) и своим типом ошибки.

use crate::repo::db::PostgresRepo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumary_dto::domain::perms::value_object::expiration::NodeExpiry;
use rumary_dto::domain::perms::value_object::group::{GroupName, GroupWeight};
use rumary_dto::domain::perms::value_object::node::{PermissionKey, SourcePriority};
use rumary_dto::domain::perms::value_object::user::UserId as PermUserId;
use rumary_dto::domain::perms::{
    AccessGrant, AccessMode, ContextSet, GroupListQuery, GroupSummary, NodeHolderType, NodeValue,
    PermissionNode, ResourceRef, GroupSnapshot,
};
use rumary_perms::{
    GroupDirectory, PermissionAdmin, PermissionError, PermissionResult, PermissionService,
    PermissionStore, ResourceAclStore, actor_outranks_target,
};
use sqlx::Row;
use sqlx::types::JsonValue;
use uuid::Uuid;

/// Рекурсивный CTE "все группы пользователя с учётом наследования" плюс
/// финальный SELECT.
///
/// Макрос, а не `const` + `format!`: sqlx 0.9 принимает только `&'static str`,
/// и это правильно — так запрос остаётся литералом, склеенным на этапе
/// компиляции, и в него физически нельзя подставить данные из запроса.
/// `$1` — user_id, дальше нумерация продолжается в теле конкретного запроса.
macro_rules! group_tree_query {
    ($tail:expr) => {
        concat!(
            r#"
            WITH RECURSIVE user_direct_groups AS (
                SELECT group_name
                FROM user_groups
                WHERE user_id = $1
                  AND (expires_at IS NULL OR expires_at > now())
            ),
            group_tree AS (
                SELECT g.id, g.name, g.weight
                FROM user_direct_groups udg
                JOIN groups g ON g.name = udg.group_name

                UNION

                SELECT pg.id, pg.name, pg.weight
                FROM group_tree gt
                JOIN group_inheritance gi ON gi.group_id = gt.id
                JOIN groups pg ON pg.name = gi.parent_name
            )
            "#,
            $tail
        )
    };
}

#[derive(sqlx::FromRow)]
struct NodeRow {
    node_key: String,
    value: bool,
    context: serde_json::Value,
    expires_at: Option<DateTime<Utc>>,
    source_priority: i32,
}

/// Представление контекста для JSONB-колонки.
fn context_to_json(context: &ContextSet) -> serde_json::Value {
    serde_json::Value::Object(
        context
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                )
            })
            .collect(),
    )
}

/// Разбор контекста из JSONB.
///
/// Невалидные пары (не-строковое значение, ключ с пробелом) отбрасываются, а
/// не роняют загрузку: это данные, уже лежащие в БД, и одна битая строка не
/// должна ломать проверку прав целиком. Отброшенное условие делает ноду более
/// широкой, поэтому пишем предупреждение в лог — тихо расширять права нельзя.
fn context_from_json(raw: &serde_json::Value, node_key: &str) -> ContextSet {
    let Some(map) = raw.as_object() else {
        return ContextSet::empty();
    };

    let pairs: Vec<(&str, &str)> = map
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key.as_str(), value)))
        .collect();

    if pairs.len() != map.len() {
        tracing::warn!(
            node_key,
            "permission node context has non-string values, skipping them"
        );
    }

    ContextSet::try_from_pairs(pairs.iter().copied()).unwrap_or_else(|err| {
        tracing::warn!(node_key, %err, "permission node has invalid context, treating as unrestricted");
        ContextSet::empty()
    })
}

/// Сборка ноды из строки БД.
///
/// Нода с невалидным ключом ПРОПУСКАЕТСЯ (возвращает None), а не превращается
/// в что-то по умолчанию: и Allow, и Deny были бы догадкой о том, чего в БД
/// не написано.
fn row_to_node(row: NodeRow) -> Option<PermissionNode> {
    let key = match PermissionKey::try_from(row.node_key.clone()) {
        Ok(key) => key,
        Err(err) => {
            tracing::warn!(
                node_key = %row.node_key,
                %err,
                "skipping permission node with invalid key"
            );
            return None;
        }
    };

    let context = context_from_json(&row.context, &row.node_key);

    Some(PermissionNode::new(
        key,
        NodeValue::from(row.value),
        context,
        row.expires_at.map(NodeExpiry::new),
        SourcePriority::new(row.source_priority),
    ))
}

/// Имя группы из БД. Как и с ключами нод, невалидное имя пропускается.
fn row_to_group_name(raw: String) -> Option<GroupName> {
    GroupName::try_from(raw.clone())
        .inspect_err(|err| {
            tracing::warn!(group = %raw, %err, "skipping group with invalid name");
        })
        .ok()
}

impl PostgresRepo {
    async fn set(
        &self,
        resource: &ResourceRef,
        grant: &AccessGrant,
        value: bool,
        mode: AccessMode,
    ) -> PermissionResult<()> {
        sqlx::query(
            "INSERT INTO resource_access (resource_type, resource_id, holder_type, holder_id, value, can_write) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (resource_type, resource_id, holder_type, holder_id) \
             DO UPDATE SET value = $5, can_write = $6, updated_at = now()",
        )
            .bind(resource.resource_type().as_str())
            .bind(resource.resource_id().as_str())
            .bind(grant.holder_type().as_str())
            .bind(grant.holder_id())
            .bind(value)
            .bind(mode.can_write())
            .execute(self.pool_ref())
            .await?;

        Ok(())
    }

    async fn has_personal_deny(
        &self,
        resource: &ResourceRef,
        user_id: PermUserId,
    ) -> PermissionResult<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT value FROM resource_access \
             WHERE resource_type = $1 AND resource_id = $2 AND holder_type = 'user' AND holder_id = $3",
        )
            .bind(resource.resource_type().as_str())
            .bind(resource.resource_id().as_str())
            .bind(user_id.to_string())
            .fetch_optional(self.pool_ref())
            .await?;

        Ok(matches!(row, Some((false,))))
    }
}

#[async_trait]
impl PermissionStore for PostgresRepo {
    async fn load_effective_nodes(
        &self,
        user_id: PermUserId,
        _ctx: &ContextSet,
    ) -> PermissionResult<Vec<PermissionNode>> {
        // Рекурсивно собираем все группы пользователя (с учётом наследования),
        // затем берём права самого пользователя + права всех этих групп.
        // weight группы становится source_priority; прямые ноды пользователя
        // получают SourcePriority::USER — заведомо выше любой группы.
        let rows: Vec<NodeRow> = sqlx::query_as(group_tree_query!(
            r#"
            SELECT
                pn.node_key,
                pn.value,
                pn.context,
                pn.expires_at,
                COALESCE(gt.weight, $2) AS source_priority
            FROM permission_nodes pn
            LEFT JOIN group_tree gt
                ON pn.holder_type = 'group' AND pn.holder_id = gt.name
            WHERE
                (pn.holder_type = 'user' AND pn.holder_id = $1::text)
                OR
                (pn.holder_type = 'group' AND pn.holder_id IN (SELECT name FROM group_tree))
            "#
        ))
        .bind(Uuid::from(user_id))
        .bind(SourcePriority::USER.get())
        .fetch_all(self.pool_ref())
        .await?;

        Ok(rows.into_iter().filter_map(row_to_node).collect())
    }

    async fn effective_group_names(&self, user_id: PermUserId) -> PermissionResult<Vec<GroupName>> {
        let rows: Vec<(String,)> = sqlx::query_as(group_tree_query!("SELECT name FROM group_tree"))
            .bind(Uuid::from(user_id))
            .fetch_all(self.pool_ref())
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(name,)| row_to_group_name(name))
            .collect())
    }

    /// Максимальный `weight` среди всех ролей пользователя (включая
    /// унаследованные). Используется для сравнения "ранга" двух пользователей —
    /// например, чтобы manager не мог удалить/забанить admin-а, даже если
    /// формально у него есть право `profile.delete`. У пользователя без единой
    /// группы — 0.
    async fn max_group_weight(&self, user_id: PermUserId) -> PermissionResult<GroupWeight> {
        let row: (Option<i32>,) =
            sqlx::query_as(group_tree_query!("SELECT MAX(weight) FROM group_tree"))
                .bind(Uuid::from(user_id))
                .fetch_one(self.pool_ref())
                .await?;

        let weight = row.0.unwrap_or(0);

        // Отрицательный вес в БД — нарушение инварианта, но не повод падать:
        // трактуем как "нет ранга", это самый безопасный вариант.
        Ok(GroupWeight::new(weight).unwrap_or_else(|_| {
            tracing::warn!(
                weight,
                "negative group weight in database, treating as no rank"
            );
            GroupWeight::NONE
        }))
    }

    /// Роли и ранг одним запросом — избавляет ACL-проверку от второго
    /// прохода по тому же рекурсивному CTE.
    async fn group_snapshot(&self, user_id: PermUserId) -> PermissionResult<GroupSnapshot> {
        let rows: Vec<(String, i32)> =
            sqlx::query_as(group_tree_query!("SELECT name, weight FROM group_tree"))
                .bind(Uuid::from(user_id))
                .fetch_all(self.pool_ref())
                .await?;

        let mut groups = Vec::with_capacity(rows.len());
        let mut max_weight = GroupWeight::NONE;

        for (name, weight) in rows {
            let Some(name) = row_to_group_name(name) else {
                continue;
            };

            if let Ok(weight) = GroupWeight::new(weight) {
                max_weight = max_weight.max(weight);
            }

            groups.push(name);
        }

        Ok(GroupSnapshot::new(groups, max_weight))
    }
}

#[async_trait]
impl GroupDirectory for PostgresRepo {
    async fn list_groups(&self, query: GroupListQuery) -> PermissionResult<Vec<GroupSummary>> {
        let rows: Vec<(String, i32)> = match query.limit {
            Some(limit) => {
                sqlx::query_as(
                    "SELECT name, weight FROM groups ORDER BY weight DESC LIMIT $1 OFFSET $2",
                )
                .bind(limit as i64)
                .bind(query.offset as i64)
                .fetch_all(self.pool_ref())
                .await?
            }
            None => {
                sqlx::query_as("SELECT name, weight FROM groups ORDER BY weight DESC")
                    .fetch_all(self.pool_ref())
                    .await?
            }
        };

        rows.into_iter()
            .map(|(name, weight)| {
                Ok(GroupSummary {
                    name: GroupName::try_from(name.as_str())?,
                    weight: GroupWeight::new(weight)?,
                })
            })
            .collect()
    }

    async fn get_group(&self, name: &GroupName) -> PermissionResult<Option<GroupSummary>> {
        let row: Option<(i32,)> = sqlx::query_as("SELECT weight FROM groups WHERE name = $1")
            .bind(name.as_str())
            .fetch_optional(self.pool_ref())
            .await?;

        row.map(|(weight,)| {
            Ok(GroupSummary {
                name: name.clone(),
                weight: GroupWeight::new(weight)?,
            })
        })
        .transpose()
    }

    /// `source_priority` для нод группы = вес самой группы — именно эта
    /// величина используется резолвером при разрешении конфликтов между
    /// группами (см. `higher_priority_source_wins_at_equal_specificity`).
    /// В `permission_nodes` она не хранится отдельно, поэтому джойним `groups`.
    async fn list_group_permissions(
        &self,
        name: &GroupName,
    ) -> PermissionResult<Vec<PermissionNode>> {
        let rows = sqlx::query(
            r#"
            SELECT pn.node_key, pn.value, pn.context, pn.expires_at, g.weight
            FROM permission_nodes pn
            JOIN groups g ON g.name = pn.holder_id
            WHERE pn.holder_type = $1 AND pn.holder_id = $2
            ORDER BY pn.node_key
            "#,
        )
        .bind(NodeHolderType::Group.as_str())
        .bind(name.as_str())
        .fetch_all(self.pool_ref())
        .await?;

        rows.into_iter()
            .map(|row| {
                let key: String = row.try_get("node_key")?;
                let allow: bool = row.try_get("value")?;
                let context_json: JsonValue = row.try_get("context")?;
                let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at")?;
                let weight: i32 = row.try_get("weight")?;

                let expires_at = expires_at.map(NodeExpiry::from);

                let context = context_from_json(&context_json, &key);

                Ok(PermissionNode::new(
                    PermissionKey::try_from(key)?,
                    NodeValue::from(allow),
                    context,
                    expires_at,
                    SourcePriority::from(weight),
                ))
            })
            .collect()
    }

    /// Участники НАПРЯМУЮ — без учёта того, что дочерние группы через
    /// наследование фактически получают те же права.
    async fn list_group_members(&self, name: &GroupName) -> PermissionResult<Vec<PermUserId>> {
        let rows: Vec<(Uuid,)> =
            sqlx::query_as("SELECT user_id FROM user_groups WHERE group_name = $1")
                .bind(name.as_str())
                .fetch_all(self.pool_ref())
                .await?;

        Ok(rows.into_iter().map(|(id,)| PermUserId::from(id)).collect())
    }

    async fn list_group_parents(&self, name: &GroupName) -> PermissionResult<Vec<GroupName>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT parent_name FROM group_inheritance
            WHERE group_id = (SELECT id FROM groups WHERE name = $1)
            "#,
        )
        .bind(name.as_str())
        .fetch_all(self.pool_ref())
        .await?;

        rows.into_iter()
            .map(|(parent,)| Ok(GroupName::try_from(parent.as_str())?))
            .collect()
    }
}

#[async_trait]
impl PermissionAdmin for PostgresRepo {
    async fn create_group(&self, name: &GroupName, weight: GroupWeight) -> PermissionResult<()> {
        sqlx::query(
            "INSERT INTO groups (name, weight) VALUES ($1, $2)",
        )
        .bind(name.as_str())
        .bind(weight.get())
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn update_group_weight(
        &self,
        name: &GroupName,
        weight: GroupWeight,
    ) -> PermissionResult<()> {
        let result = sqlx::query("UPDATE groups SET weight = $2 WHERE name = $1")
            .bind(name.as_str())
            .bind(weight.get())
            .execute(self.pool_ref())
            .await?;

        if result.rows_affected() == 0 {
            return Err(PermissionError::StoreError(sqlx::Error::InvalidArgument(
                format!("group '{}' does not exist", name.as_str()),
            )));
        }

        Ok(())
    }

    /// Удаление группы: `group_inheritance` уйдёт по FK CASCADE, а
    /// `permission_nodes`/`user_groups` ссылаются на имя группы текстом, без
    /// FK — их чистим руками, иначе останутся ноды-сироты, которые "оживут"
    /// при создании группы с тем же именем.
    async fn delete_group(&self, name: &GroupName) -> PermissionResult<()> {
        let mut tx = self.pool_ref().begin().await?;

        sqlx::query("DELETE FROM permission_nodes WHERE holder_type = $1 AND holder_id = $2")
            .bind(NodeHolderType::Group.as_str())
            .bind(name.as_str())
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM user_groups WHERE group_name = $1")
            .bind(name.as_str())
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM group_inheritance WHERE parent_name = $1")
            .bind(name.as_str())
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM groups WHERE name = $1")
            .bind(name.as_str())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn add_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
        context: &ContextSet,
    ) -> PermissionResult<()> {
        // Цикл в наследовании не сломает рекурсивный CTE (UNION отсекает
        // повторы), но делает граф ролей бессмысленным, поэтому отклоняем сразу.
        if group == parent {
            return Err(PermissionError::StoreError(sqlx::Error::InvalidArgument(
                "group cannot inherit from itself".to_owned(),
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO group_inheritance (group_id, parent_name, context)
            SELECT id, $2, $3 FROM groups WHERE name = $1
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(group.as_str())
        .bind(parent.as_str())
        .bind(context_to_json(context))
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn remove_group_parent(
        &self,
        group: &GroupName,
        parent: &GroupName,
    ) -> PermissionResult<()> {
        sqlx::query(
            r#"
            DELETE FROM group_inheritance
            WHERE parent_name = $2
              AND group_id = (SELECT id FROM groups WHERE name = $1)
            "#,
        )
        .bind(group.as_str())
        .bind(parent.as_str())
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    /// ON CONFLICT по (holder_type, holder_id, node_key, context) —
    /// повторная выдача того же права перезаписывает значение вместо того,
    /// чтобы плодить дубли строк с противоречивыми value.
    async fn set_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
        value: NodeValue,
        context: &ContextSet,
    ) -> PermissionResult<()> {
        sqlx::query(
            "INSERT INTO permission_nodes (holder_type, holder_id, node_key, value, context) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (holder_type, holder_id, node_key, context) \
             DO UPDATE SET value = $4",
        )
        .bind(NodeHolderType::Group.as_str())
        .bind(group.as_str())
        .bind(key.as_str())
        .bind(value.is_allow())
        .bind(context_to_json(context))
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn revoke_group_permission(
        &self,
        group: &GroupName,
        key: &PermissionKey,
    ) -> PermissionResult<()> {
        sqlx::query(
            "DELETE FROM permission_nodes \
             WHERE holder_type = $1 AND holder_id = $2 AND node_key = $3",
        )
        .bind(NodeHolderType::Group.as_str())
        .bind(group.as_str())
        .bind(key.as_str())
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn set_user_permission(
        &self,
        user_id: PermUserId,
        key: &PermissionKey,
        value: NodeValue,
        context: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> PermissionResult<()> {
        sqlx::query(
            "INSERT INTO permission_nodes (holder_type, holder_id, node_key, value, context, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (holder_type, holder_id, node_key, context) \
             DO UPDATE SET value = $4, expires_at = $6",
        )
            .bind(NodeHolderType::User.as_str())
            .bind(user_id.to_string())
            .bind(key.as_str())
            .bind(value.is_allow())
            .bind(context_to_json(context))
            .bind(expires_at.map(NodeExpiry::get))
            .execute(self.pool_ref())
            .await?;

        Ok(())
    }

    async fn revoke_user_permission(
        &self,
        user_id: PermUserId,
        key: &PermissionKey,
    ) -> PermissionResult<()> {
        sqlx::query(
            "DELETE FROM permission_nodes \
             WHERE holder_type = $1 AND holder_id = $2 AND node_key = $3",
        )
        .bind(NodeHolderType::User.as_str())
        .bind(user_id.to_string())
        .bind(key.as_str())
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn add_user_to_group(
        &self,
        user_id: PermUserId,
        group: &GroupName,
        context: &ContextSet,
        expires_at: Option<NodeExpiry>,
    ) -> PermissionResult<()> {
        sqlx::query(
            "INSERT INTO user_groups (user_id, group_name, context, expires_at) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id, group_name, context) DO UPDATE SET expires_at = $4",
        )
        .bind(Uuid::from(user_id))
        .bind(group.as_str())
        .bind(context_to_json(context))
        .bind(expires_at.map(NodeExpiry::get))
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    async fn remove_user_from_group(
        &self,
        user_id: PermUserId,
        group: &GroupName,
    ) -> PermissionResult<()> {
        sqlx::query("DELETE FROM user_groups WHERE user_id = $1 AND group_name = $2")
            .bind(Uuid::from(user_id))
            .bind(group.as_str())
            .execute(self.pool_ref())
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ResourceAclStore for PostgresRepo {
    /// Выдать доступ (allow) group, конкретному пользователю или "рангу и выше".
    async fn grant(
        &self,
        resource: &ResourceRef,
        grant: &AccessGrant,
        mode: AccessMode,
    ) -> PermissionResult<()> {
        self.set(resource, grant, true, mode).await
    }

    /// Явно ЗАПРЕТИТЬ доступ конкретному пользователю к этому ресурсу,
    /// даже если его роль в остальном разрешает.
    ///
    /// ВАЖНО: это единственный правильный вход для запрета — он требует
    /// `actor_id` и проверяет, что actor "выше" по рангу, чем target
    /// (`actor_outranks_target`). Именно так работает требование
    /// "запрет должен работать только сверху-вниз": writer не может
    /// запретить доступ другому writer или admin-у, а admin — writer-у может.
    /// Прямого доступа к `set(..., value=false, ...)` наружу нет намеренно.
    async fn deny_user(
        &self,
        store: &dyn PermissionStore,
        resource: &ResourceRef,
        actor_id: PermUserId,
        target_user_id: PermUserId,
    ) -> PermissionResult<()> {
        if !actor_outranks_target(store, actor_id, target_user_id).await? {
            return Err(PermissionError::InsufficientRank(
                "actor does not outrank target — cannot deny access",
            ));
        }

        self.set(
            resource,
            &AccessGrant::User(target_user_id),
            false,
            AccessMode::ReadOnly,
        )
        .await
    }

    /// Отозвать (удалить) запись целиком — и allow, и deny, снимает override.
    async fn revoke(&self, resource: &ResourceRef, grant: &AccessGrant) -> PermissionResult<()> {
        sqlx::query(
            "DELETE FROM resource_access \
             WHERE resource_type = $1 AND resource_id = $2 AND holder_type = $3 AND holder_id = $4",
        )
        .bind(resource.resource_type().as_str())
        .bind(resource.resource_id().as_str())
        .bind(grant.holder_type().as_str())
        .bind(grant.holder_id())
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    /// Вызывать при удалении самого ресурса — тут нет FK CASCADE (см. миграцию),
    /// поэтому чистим ACL руками.
    async fn revoke_all_for_resource(&self, resource: &ResourceRef) -> PermissionResult<()> {
        sqlx::query("DELETE FROM resource_access WHERE resource_type = $1 AND resource_id = $2")
            .bind(resource.resource_type().as_str())
            .bind(resource.resource_id().as_str())
            .execute(self.pool_ref())
            .await?;

        Ok(())
    }

    /// Доступен ли конкретный ресурс пользователю. `groups` — снимок ролей и
    /// ранга (см. `PermissionStore::group_snapshot`), ранг нужен для проверки
    /// MinWeight-грантов.
    async fn is_allowed(
        &self,
        resource: &ResourceRef,
        user_id: PermUserId,
        groups: &GroupSnapshot,
    ) -> PermissionResult<bool> {
        // Шаг 1: личная запись — решает окончательно, allow или deny.
        let personal: Option<(bool,)> = sqlx::query_as(
            "SELECT value FROM resource_access \
             WHERE resource_type = $1 AND resource_id = $2 AND holder_type = 'user' AND holder_id = $3",
        )
            .bind(resource.resource_type().as_str())
            .bind(resource.resource_id().as_str())
            .bind(user_id.to_string())
            .fetch_optional(self.pool_ref())
            .await?;

        if let Some((value,)) = personal {
            return Ok(value);
        }

        // Шаг 2: любая подходящая group/min_weight запись с value=true.
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM resource_access \
             WHERE resource_type = $1 AND resource_id = $2 AND value = true \
               AND ( (holder_type = 'group' AND holder_id = ANY($3)) \
                  OR (holder_type = 'min_weight' AND holder_id::int <= $4) ) \
             LIMIT 1",
        )
        .bind(resource.resource_type().as_str())
        .bind(resource.resource_id().as_str())
        .bind(groups.group_names())
        .bind(groups.max_weight().get())
        .fetch_optional(self.pool_ref())
        .await?;

        Ok(row.is_some())
    }

    /// То же самое, но сначала проверяет RBAC-обход по конвенции
    /// `{resource_type}.bypass_acl` (owner с `*`, admin с `configuration.*`
    /// для resource_type=configuration и т.п.).
    ///
    /// Порядок здесь обратный к прежнему: СНАЧАЛА личный deny, и только потом
    /// bypass. Личный запрет — это адресное решение админа по конкретному
    /// человеку и ресурсу, а bypass — общее правило роли; общее правило не
    /// должно молча отменять адресное. Owner, которому нужно вернуть себе
    /// доступ, снимает свой deny через `revoke`, а не обходит его.
    async fn is_allowed_with_bypass(
        &self,
        perms: &PermissionService,
        resource: &ResourceRef,
        user_id: PermUserId,
        groups: &GroupSnapshot,
    ) -> PermissionResult<bool> {
        if self.has_personal_deny(resource, user_id).await? {
            return Ok(false);
        }

        let bypass_key = resource.resource_type().bypass_acl_key();
        if perms
            .check(user_id, &bypass_key, &ContextSet::empty())
            .await
        {
            return Ok(true);
        }

        self.is_allowed(resource, user_id, groups).await
    }
}
