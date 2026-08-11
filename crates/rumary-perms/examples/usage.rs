//! Полный сценарий: кастомная роль "writer", публичный и приватный ресурс,
//! точечный запрет сверху вниз по рангу.
//!
//! Запуск: `cargo run -p rumary-perms --example usage`
//!
//! Пример работает на `InMemoryStore` и не требует БД — ACL-часть (`ResourceAcl`)
//! требует Postgres, поэтому здесь показана RBAC-часть и проверка ранга.

use rumary_perms::domain::value_object::group::{GroupName, GroupWeight};
use rumary_perms::domain::value_object::node::{PermissionKey, SourcePriority};
use rumary_perms::domain::value_object::user::UserId;
use rumary_perms::domain::{ContextSet, NodeValue, PermissionNode};
use rumary_perms::{
    InMemoryStore, PermissionError, PermissionService, actor_outranks_target, require_outranks,
};
use uuid::Uuid;

fn key(raw: &str) -> PermissionKey {
    PermissionKey::try_from(raw).expect("valid permission key")
}

fn group(name: &str, weight: i32) -> (GroupName, GroupWeight) {
    (
        GroupName::try_from(name).expect("valid group name"),
        GroupWeight::new(weight).expect("valid weight"),
    )
}

/// Бизнес-функция, которая сама проверяет право через `require` и прокидывает
/// ошибку наверх через `?`.
async fn delete_configuration(
    perms: &PermissionService,
    actor_id: UserId,
    config_id: Uuid,
) -> Result<(), PermissionError> {
    let ctx = ContextSet::try_from_pairs([("tenant", "acme")]).expect("valid context");
    perms
        .require(actor_id, &key("configuration.delete"), &ctx)
        .await?;

    println!("configuration {config_id} deleted by {actor_id}");
    Ok(())
}

/// Место, где нужен просто bool без ошибки — например, чтобы решить,
/// показывать ли поле в ответе.
async fn can_share_configuration(perms: &PermissionService, actor_id: UserId) -> bool {
    perms
        .check(actor_id, &key("configuration.share"), &ContextSet::empty())
        .await
}

#[tokio::main]
async fn main() {
    let tenant_ctx = ContextSet::try_from_pairs([("tenant", "acme")]).expect("valid context");

    let admin_id = UserId::from(Uuid::new_v4());
    let writer_id = UserId::from(Uuid::new_v4());

    let mut store = InMemoryStore::new();

    // --- admin: полный доступ к конфигурациям через wildcard группы ---
    store
        .set_groups(admin_id, vec![group("admin", 30)])
        .set_nodes(
            admin_id,
            vec![PermissionNode::permanent(
                key("configuration.*"),
                NodeValue::Allow,
                SourcePriority::new(30),
            )],
        );

    // --- writer: может создавать и шарить, но НЕ удалять ---
    // Обратите внимание на явный Deny: он специфичнее группового wildcard,
    // поэтому перебивает его — так же, как в LuckPerms.
    store.set_groups(writer_id, vec![group("writer", 15)]).set_nodes(
        writer_id,
        vec![
            PermissionNode::new(
                key("configuration.*"),
                NodeValue::Allow,
                tenant_ctx.clone(),
                None,
                SourcePriority::new(15),
            ),
            PermissionNode::permanent(
                key("configuration.delete"),
                NodeValue::Deny,
                SourcePriority::USER,
            ),
        ],
    );

    let perms = PermissionService::new(store.clone());

    // admin удаляет — проходит
    delete_configuration(&perms, admin_id, Uuid::new_v4())
        .await
        .expect("admin may delete configurations");

    // writer удаляет — явный запрет перебивает групповой wildcard
    match delete_configuration(&perms, writer_id, Uuid::new_v4()).await {
        Err(PermissionError::Denied(denied_key)) => {
            println!("writer denied: {denied_key}");
        }
        other => panic!("expected explicit deny, got {other:?}"),
    }

    // writer шарит — групповой wildcard действует, но только в своём контексте
    println!(
        "writer can share (no context): {}",
        can_share_configuration(&perms, writer_id).await
    );
    println!(
        "writer can share (tenant=acme): {}",
        perms
            .check(writer_id, &key("configuration.share"), &tenant_ctx)
            .await
    );

    // --- Проверка ранга: отдельная от проверки права ---
    println!(
        "admin outranks writer: {}",
        actor_outranks_target(&store, admin_id, writer_id)
            .await
            .expect("store available")
    );

    // writer не выше другого writer — запрет "в сторону" отклоняется
    let peer_id = UserId::from(Uuid::new_v4());
    let mut peer_store = store.clone();
    peer_store.set_groups(peer_id, vec![group("writer", 15)]);

    assert!(require_outranks(&peer_store, writer_id, peer_id).await.is_err());
    println!("writer cannot act on peer writer — as intended");
}
