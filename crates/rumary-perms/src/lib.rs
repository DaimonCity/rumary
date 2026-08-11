//! Проверка прав в стиле LuckPerms: wildcard-ноды, контексты, наследование
//! групп, веса, временные права — плюс resource-level ACL поверх того же
//! хранилища.
//!
//! Доменные типы (`PermissionNode`, `ContextSet`, `Tristate`, `AccessGrant`,
//! value object-ы) живут в `rumary-dto` — этот крейт содержит только логику:
//! резолвинг, кэш, трейты хранилища и SQL-часть ACL.
//!
//! Точки входа:
//! - `PermissionService` — RBAC-проверка ("может ли группа такое вообще"),
//! - `ResourceAcl` — доступ к конкретной записи ("к этому ли ресурсу"),
//! - `require_outranks` — ранг ("выше ли actor, чем цель действия").
//!
//! Для действий над другим пользователем нужны все три.

mod admin;
mod error;
mod hierarchy;
mod resolver;
mod resource_acl;
mod service;
mod store;
mod group_directory;

pub use admin::PermissionAdmin;
pub use error::{PermissionError, PermissionResult};
pub use hierarchy::{actor_outranks_target, require_outranks};
pub use resolver::{resolve, resolve_at};
pub use resource_acl::{ResourceAclStore};
pub use service::PermissionService;
pub use store::{FailingStore, InMemoryStore, PermissionStore};
pub use group_directory::GroupDirectory;

/// Реэкспорт доменных типов, чтобы вызывающему коду не приходилось тянуть
/// `rumary_dto::domain::perms::...` рядом с каждым использованием сервиса.
pub mod domain {
    pub use rumary_dto::domain::perms::*;
}
