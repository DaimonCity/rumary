//! Прикладной фасад над `rumary-perms`.
//!
//! Отвечает на вопрос "можно ли этому пользователю сделать это с этим
//! ресурсом", объединяя три независимые проверки:
//! 1. RBAC — может ли роль выполнять действие такого рода (`configuration.get`);
//! 2. ACL — открыт ли доступ к конкретной записи (или ресурс публичный);
//! 3. ранг — выше ли actor цели, для действий над другим пользователем.

use crate::error::AppError;
use rumary_dto::domain::api::share_target::ShareTarget;
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::value_object::resource::{ResourceId, ResourceType};
use rumary_dto::domain::perms::value_object::user::UserId as PermsUserId;
use rumary_dto::domain::perms::{AccessGrant, AccessMode, ContextSet, ResourceRef};
use rumary_perms::{
    require_outranks, PermissionService, ResourceAclStore,
};
use std::sync::Arc;

/// Действия над ресурсом — из них собираются ключи прав вида
/// `configuration.get`. Enum вместо строк на месте вызова: опечатка
/// в `"configuraton.get"` иначе прошла бы компиляцию и молча запретила всё.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAction {
    Get,
    List,
    Create,
    Update,
    Delete,
    Share,
    Download,
    Ban,
    BanPermanent,
    Unban,
}

impl ResourceAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::List => "list",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Share => "share",
            Self::Download => "download",
            Self::Ban => "ban",
            Self::BanPermanent => "ban.permanent",
            Self::Unban => "unban",
        }
    }
}

pub struct PermissionsFacade {
    perms: Arc<PermissionService>,
    acl: Arc<dyn ResourceAclStore>,
}

impl PermissionsFacade {
    pub fn new(perms: PermissionService, acl: impl ResourceAclStore) -> Self {
        Self {
            perms: Arc::new(perms),
            acl: Arc::new(acl),
        }
    }

    pub fn from_arc(perms: Arc<PermissionService>, acl: Arc<dyn ResourceAclStore>) -> Self {
        Self { perms, acl }
    }

    pub fn service(&self) -> &PermissionService {
        &self.perms
    }

    pub fn acl(&self) -> &Arc<dyn ResourceAclStore> {
        &self.acl
    }

    /// RBAC-гейт для действий, не привязанных к конкретной записи:
    /// `configuration.create`, `configuration.list`.
    ///
    /// Ошибку хранилища НЕ проглатываем: `require` вернёт `StoreError`, и
    /// хендлер ответит 500, а не 403 — иначе сбой БД выглядит как отказ в
    /// правах и уводит отладку в неверную сторону.
    pub async fn require_action(
        &self,
        actor_id: PermsUserId,
        resource_type: &ResourceType,
        action: ResourceAction,
        ctx: &ContextSet,
    ) -> Result<(), AppError> {
        let key = resource_type.action_key(action.as_str()).map_err(|err| {
            AppError::Internal(format!(
                "malformed permission key for {resource_type}: {err}"
            ))
        })?;

        self.perms.require(actor_id, &key, ctx).await?;

        Ok(())
    }

    /// Полная проверка доступа к КОНКРЕТНОЙ записи: RBAC-гейт + ACL.
    ///
    /// `is_public` — свойство самого ресурса (колонка `is_public`), которое
    /// вызывающий код уже прочитал вместе с записью. Публичный ресурс
    /// проходит ACL-проверку без похода в `resource_access`, но RBAC-гейт
    /// остаётся: "публичный" значит "доступен всем, кому вообще можно читать
    /// ресурсы этого типа", а не "доступен анониму".
    pub async fn require_resource_access(
        &self,
        actor_id: PermsUserId,
        resource: &ResourceRef,
        action: ResourceAction,
        is_public: bool,
        ctx: &ContextSet,
    ) -> Result<(), AppError> {
        self.require_action(actor_id, resource.resource_type(), action, ctx)
            .await?;

        if is_public {
            return Ok(());
        }

        let roles = self.perms.store().group_snapshot(actor_id).await?;

        let allowed = self
            .acl
            .is_allowed_with_bypass(&self.perms, resource, actor_id, &roles)
            .await?;

        if allowed {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("no access to {resource}")))
        }
    }

    /// bool-вариант для мест, где отказ — это не ошибка, а ветка логики
    /// (отфильтровать список, спрятать поле в ответе).
    pub async fn can_access_resource(
        &self,
        actor_id: PermsUserId,
        resource: &ResourceRef,
        action: ResourceAction,
        is_public: bool,
        ctx: &ContextSet,
    ) -> bool {
        self.require_resource_access(actor_id, resource, action, is_public, ctx)
            .await
            .is_ok()
    }

    /// Действие над другим пользователем (ban, delete, смена чужих прав):
    /// RBAC-гейт И проверка ранга. Обе обязательны — право `profile.delete`
    /// не должно позволять manager-у удалить admin-а.
    pub async fn require_action_on_user(
        &self,
        actor_id: PermsUserId,
        target_id: PermsUserId,
        resource_type: &ResourceType,
        action: ResourceAction,
        ctx: &ContextSet,
    ) -> Result<(), AppError> {
        self.require_action(actor_id, resource_type, action, ctx)
            .await?;

        require_outranks(self.perms.store().as_ref(), actor_id, target_id).await?;

        Ok(())
    }

    /// Регистрация только что созданного ресурса в ACL: автор получает
    /// личный доступ на запись, дальше — ноль или больше целей из
    /// `share_with`, каждая на чтение.
    ///
    /// Вызывать сразу после вставки записи: иначе ресурс существует, но
    /// недоступен даже автору.
    pub async fn register_created_resource(
        &self,
        author_id: PermsUserId,
        resource: &ResourceRef,
        share_with: &[ShareTarget],
    ) -> Result<(), AppError> {
        self.acl
            .grant(resource, &AccessGrant::User(author_id), AccessMode::ReadWrite)
            .await?;

        for target in share_with {
            self.share_one(author_id, resource, target, AccessMode::ReadOnly).await?;
        }

        Ok(())
    }

    /// Расшарить уже существующий ресурс дополнительной цели — в любой
    /// момент его жизни, не только при создании.
    pub async fn share_resource(
        &self,
        author_id: PermsUserId,
        resource: &ResourceRef,
        target: &ShareTarget,
        mode: AccessMode,
    ) -> Result<(), AppError> {
        self.share_one(author_id, resource, target, mode).await
    }

    /// Убрать ранее выданный доступ. Не путать с `deny_user_access` —
    /// `unshare` просто снимает положительный грант, `deny_user_access`
    /// ставит явный запрет поверх того, что разрешает роль.
    pub async fn unshare_resource(
        &self,
        resource: &ResourceRef,
        target: &ShareTarget,
    ) -> Result<(), AppError> {
        match target {
            ShareTarget::Peers => {
                // "Peers" не хранит вес сам по себе — при unshare нужен
                // явный порог, иначе неясно, чей ранг снимать.
                return Err(AppError::Internal(
                    "cannot unshare ShareTarget::Peers without explicit rank; use MinRank".into(),
                ));
            }
            ShareTarget::MinRank(weight) => {
                self.acl.revoke(resource, &AccessGrant::MinWeight(*weight)).await?;
            }
            ShareTarget::Role(name) => {
                self.acl.revoke(resource, &AccessGrant::Group(name.clone())).await?;
            }
            ShareTarget::Users(ids) => {
                for id in ids {
                    self.acl.revoke(resource, &AccessGrant::User(*id)).await?;
                }
            }
        }
        Ok(())
    }

    async fn share_one(
        &self,
        author_id: PermsUserId,
        resource: &ResourceRef,
        target: &ShareTarget,
        mode: AccessMode,
    ) -> Result<(), AppError> {
        match target {
            ShareTarget::Peers => {
                let roles = self.perms.store().group_snapshot(author_id).await?;
                self.acl
                    .grant(resource, &AccessGrant::MinWeight(roles.max_weight()), mode)
                    .await?;
            }
            ShareTarget::MinRank(weight) => {
                self.acl.grant(resource, &AccessGrant::MinWeight(*weight), mode).await?;
            }
            ShareTarget::Role(name) => {
                self.acl.grant(resource, &AccessGrant::Group(name.clone()), mode).await?;
            }
            ShareTarget::Users(ids) => {
                for id in ids {
                    self.acl.grant(resource, &AccessGrant::User(*id), mode).await?;
                }
            }
        }
        Ok(())
    }

    /// Зачистка при удалении ресурса — на общей ACL-таблице нет FK CASCADE,
    /// поэтому записи нужно удалять руками.
    pub async fn cleanup_deleted_resource(&self, resource: &ResourceRef) -> Result<(), AppError> {
        self.acl.revoke_all_for_resource(resource).await?;
        Ok(())
    }

    /// Точечный запрет доступа конкретному пользователю — только сверху вниз
    /// по рангу, см. `ResourceAcl::deny_user`.
    pub async fn deny_user_access(
        &self,
        actor_id: PermsUserId,
        target_id: PermsUserId,
        resource: &ResourceRef,
    ) -> Result<(), AppError> {
        self.acl
            .deny_user(self.perms.store().as_ref(), resource, actor_id, target_id)
            .await?;

        Ok(())
    }
}

/// Типы ресурсов приложения — валидируются один раз при старте, а не на
/// каждый запрос.
#[derive(Clone)]
pub struct ResourceTypes {
    pub configuration: ResourceType,
    pub instance: ResourceType,
    pub profile: ResourceType,
}

impl ResourceTypes {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            configuration: Self::parse("configuration")?,
            instance: Self::parse("instance")?,
            profile: Self::parse("profile")?,
        })
    }

    fn parse(raw: &'static str) -> Result<ResourceType, AppError> {
        ResourceType::try_from(raw)
            .map_err(|err| AppError::Configuration(format!("invalid resource type {raw}: {err}")))
    }
}

/// Ссылка на ресурс из uuid — самый частый случай в хендлерах.
pub fn resource_ref(resource_type: &ResourceType, id: uuid::Uuid) -> ResourceRef {
    ResourceRef::new(resource_type.clone(), ResourceId::from(id))
}

/// Ключ права из статической строки. Паникует только на литералах, которые
/// разработчик написал неправильно — то есть падает на первом же тесте, а не
/// в продакшене на живом запросе.
pub fn static_key(raw: &'static str) -> PermissionKey {
    PermissionKey::try_from(raw).expect("static permission key must be valid")
}

/// Тип, который держит `PermissionsFacade` в состоянии приложения.
pub type SharedPermissions = Arc<PermissionsFacade>;
