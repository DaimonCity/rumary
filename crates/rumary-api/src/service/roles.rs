#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::repo::repository::{RightsRepository, RolesRepository};
use crate::service::right::Rights;
use rumary_dto::domain::api::{NewRole, RightKey, Role, RoleError, RoleId, UpdateRoleDb};
use rumary_dto::domain::configuration::ConfigurationId;
use rumary_dto::domain::instance::InstanceId;
use rumary_dto::dto::api::response::role::GetRoleResponse;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub struct RoleService {
    roles_repo: Arc<dyn RolesRepository<Error = AppError>>,
    right_repo: Arc<dyn RightsRepository<Error = AppError>>,
    roles_ids: Vec<RoleId>,
    roles: Vec<Role>,
    rights: Rights,
    channel: Option<tokio::sync::mpsc::Receiver<Rights>>,
}

impl RoleService {
    pub async fn new(
        roles_repo: Arc<dyn RolesRepository<Error = AppError>>,
        right_repo: Arc<dyn RightsRepository<Error = AppError>>,
        channel: tokio::sync::mpsc::Receiver<Rights>,
    ) -> AppResult<Self> {
        let rows = roles_repo.list_roles().await?;
        let roles_ids = rows.iter().map(|r| r.id).collect::<Vec<_>>();
        let roles = rows.into_iter().map(|r| r.into()).collect::<Vec<Role>>();
        let rights = right_repo.get_rights().await?;

        let mut service = Self::init(roles_ids, roles, rights, roles_repo, right_repo, channel);

        match service.init_root().await {
            Ok(()) | Err(AppError::RoleError(RoleError::Exists(_))) => {}
            Err(err) => return Err(err),
        }

        Ok(service)
    }

    fn init(
        roles_ids: Vec<RoleId>,
        roles: Vec<Role>,
        rights: Rights,
        roles_repo: Arc<dyn RolesRepository<Error = AppError>>,
        right_repo: Arc<dyn RightsRepository<Error = AppError>>,
        channel: tokio::sync::mpsc::Receiver<Rights>,
    ) -> Self {
        Self {
            roles_repo,
            right_repo,
            channel: Some(channel),
            rights,
            roles_ids,
            roles,
        }
    }

    async fn init_root(&mut self) -> AppResult<()> {
        let rights = &self.rights;
        let mut role = Role::new("root", &rights.rights_ids(), &rights.default_values());
        role.allow_all();
        self.persist_new_role(role).await?;
        Ok(())
    }

    async fn init_user(&mut self) -> AppResult<()> {
        let rid = self.create_role("user").await?;
        let rights_ids = self.rights.rights_ids();
        let def = self.rights.default_values();

        let role = self.get_mut_role(&rid)?;
        role.set_rights(rights_ids.into_iter().zip(def).collect());
        Ok(())
    }

    pub async fn create_role(&mut self, name: &str) -> AppResult<RoleId> {
        if self.roles.iter().filter(|r| r.name() == name).count() > 0 {
            return Err(AppError::RoleError(RoleError::Exists(format!(
                "The {} alredy exists",
                name
            ))));
        }
        let role = Role::new(
            name,
            &self.rights.rights_ids(),
            &self.rights.default_values(),
        );
        self.persist_new_role(role).await
    }

    async fn persist_new_role(&mut self, role: Role) -> AppResult<RoleId> {
        if self
            .roles
            .iter()
            .any(|existing| existing.name() == role.name())
        {
            return Err(AppError::RoleError(RoleError::Exists(format!(
                "The {} already exists",
                role.name()
            ))));
        }

        let rid = self.increment_roles_ids();

        let new_role = NewRole::new(rid, role);

        let role = self.roles_repo.create_role(new_role).await?;
        self.roles.push(role.into());

        Ok(rid)
    }

    pub fn insert_role(&mut self, role: Role) {
        self.roles.push(role);
        self.increment_roles_ids();
    }

    pub fn insert_roles(&mut self, roles: Box<[Role]>) {
        let _ = roles.into_iter().map(|role| self.insert_role(role));
    }

    pub async fn update_role<'a>(
        &mut self,
        rid: RoleId,
        allow_keys: &[RightKey<'a>],
        remove_keys: &[RightKey<'a>],
    ) -> AppResult<RoleId> {
        let (allow_rights, remove_rights) = {
            let rights_handle = &self.rights;

            let allow = allow_keys
                .iter()
                .map(|k| rights_handle.get_right(k))
                .collect::<AppResult<Vec<_>>>()?;

            let remove = remove_keys
                .iter()
                .map(|k| rights_handle.get_right(k))
                .collect::<AppResult<Vec<_>>>()?;

            (allow, remove)
        };

        let role = self.get_mut_role(&rid)?;

        role.add_rights(&allow_rights);
        role.remove_rights(&remove_rights);

        self.persist_update_role(rid).await?;

        Ok(rid)
    }

    async fn persist_update_role(&self, rid: RoleId) -> AppResult<RoleId> {
        let update_role = {
            let role = self.get_role(&rid)?.clone();
            UpdateRoleDb::new(rid, role)
        };
        let _ = self.roles_repo.update_role(update_role).await?;
        Ok(rid)
    }

    pub fn get_role_info(&self, rid: RoleId) -> AppResult<GetRoleResponse> {
        let role = self.get_role(&rid)?;
        let get_response = GetRoleResponse {
            id: rid.into(),
            name: role.name().into(),
            rights: role.rights_cloned().into(),
        };

        Ok(get_response)
    }
    pub fn list_role(&self) -> AppResult<Vec<GetRoleResponse>> {
        // self.roles_repo.list_roles() also can be used
        self.roles_ids
            .iter()
            .map(|rid| self.get_role_info(*rid))
            .collect::<AppResult<Vec<_>>>()
    }

    fn get_mut_role(&mut self, rid: &RoleId) -> AppResult<&mut Role> {
        let index = self.get_index(rid)?;
        self.roles
            .get_mut(index)
            .ok_or(AppError::NotFound("no such role".to_string()))
    }
    fn get_role(&self, rid: &RoleId) -> AppResult<&Role> {
        let index = self.get_index(rid)?;
        self.roles
            .get(index)
            .ok_or(AppError::NotFound("no such role".to_string()))
    }

    pub fn remove_role(&mut self, rid: RoleId) -> AppResult<()> {
        let index = self.get_index(&rid)?;

        self.roles.remove(index);
        self.roles_ids.remove(index);
        Ok(())
    }

    pub async fn is_available_action<'a>(
        &self,
        user_role_id: &RoleId,
        right_key: &RightKey<'a>,
    ) -> AppResult<bool> {
        let role = self.get_role(user_role_id)?;
        let right_id = self.rights.get_right(right_key)?;
        let roles_rights = role.rights();

        Ok(roles_rights.get(&right_id).copied().unwrap_or(false))
    }
    fn get_index(&self, rid: &RoleId) -> AppResult<usize> {
        self.roles_ids
            .iter()
            .position(|i| i == rid)
            .ok_or(AppError::Internal(
                "Cannot get index with RoleID does not exist".to_string(),
            ))
    }
    fn increment_roles_ids(&mut self) -> RoleId {
        let role_id = self
            .roles_ids
            .iter()
            .max_by(|a, b| a.cmp(b))
            .unwrap_or(&RoleId::start())
            .increment();
        self.roles_ids.push(role_id);
        role_id
    }
    fn generate_permissions(
        ids: &[impl Display],
        actions: &[&str],
        key_word: &str,
    ) -> Vec<RightKey<'static>> {
        ids.iter()
            .flat_map(|id| {
                actions
                    .iter()
                    .map(move |action| RightKey::new(format!("{}.{}.{}", key_word, id, action)))
            })
            .collect()
    }

    pub fn take_rights_channel(&mut self) -> Option<tokio::sync::mpsc::Receiver<Rights>> {
        self.channel.take()
    }

    pub async fn apply_rights(&mut self, new_rights: Rights) {
        let rights_ids = new_rights.rights_ids();
        let default_values = new_rights.default_values();

        for role in &mut self.roles {
            role.reconcile_rights(&rights_ids, &default_values);
        }

        self.rights = new_rights;
    }

    pub async fn instance_list(&self, roles: &[RoleId]) -> AppResult<Vec<InstanceId>> {
        self.list_from_rights::<InstanceId, Uuid>(roles).await
    }
    pub async fn configuration_list(&self, roles: &[RoleId]) -> AppResult<Vec<ConfigurationId>> {
        self.list_from_rights::<ConfigurationId, Uuid>(roles).await
    }

    pub async fn role_list(&self, roles: &[RoleId]) -> AppResult<Vec<RoleId>> {
        self.list_from_rights::<RoleId, usize>(roles).await
    }

    async fn list_from_rights<T, S>(&self, roles: &[RoleId]) -> AppResult<Vec<T>>
    where
        T: From<S> + std::cmp::PartialEq + EntityName,
        S: FromStr,
    {
        let mut entiers: Vec<T> = Vec::new();
        let part = T::ENTITY_NAME;

        let instance_gets: Vec<RightKey<'static>> = self
            .rights
            .rights_keys()
            .into_iter()
            .filter(|k| k.as_str().contains(part) && k.as_str().contains("get"))
            .collect();

        for get_key in instance_gets {
            let entier: T = S::from_str(get_key.as_str().split('.').collect::<Vec<&str>>()[1])
                .map_err(|_err| AppError::Internal("Parsing error".to_string()))?
                .into();
            for role in roles {
                if self.is_available_action(role, &get_key).await? && !entiers.contains(&entier) {
                    entiers.push(entier);
                    break;
                }
            }
        }

        Ok(entiers)
    }

    pub async fn add_right(
        &self,
        right_key: RightKey<'static>,
        default_value: bool,
    ) -> AppResult<()> {
        self.right_repo.add_right(right_key, default_value).await
    }

    pub async fn add_rights(
        &self,
        right_keys: &[RightKey<'static>],
        default_value: &[bool],
    ) -> AppResult<()> {
        self.right_repo.add_rights(right_keys, default_value).await
    }

    pub async fn remove_right(&self, right_key: RightKey<'static>) -> AppResult<()> {
        self.right_repo.remove_right(right_key).await
    }
    pub async fn remove_rights(&self, right_key: &[RightKey<'static>]) -> AppResult<()> {
        self.right_repo.remove_rights(right_key).await
    }
}

pub trait EntityName {
    const ENTITY_NAME: &'static str;
}

impl EntityName for InstanceId {
    const ENTITY_NAME: &'static str = "instance";
}

impl EntityName for ConfigurationId {
    const ENTITY_NAME: &'static str = "configuration";
}

impl EntityName for RoleId {
    const ENTITY_NAME: &'static str = "role";
}
