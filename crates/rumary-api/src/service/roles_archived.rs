#![allow(dead_code)]
use crate::error::{AppError, AppResult};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RightId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RoleId(usize);
#[derive(PartialEq)]
pub struct RightKey(String);
pub struct RoleService {
    roles_ids: Vec<RoleId>,
    roles: Vec<Role>,
    rights: Rights,
}

impl RoleId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

pub struct Rights {
    rights_ids: Vec<RightId>,   // unique -> private
    rights_keys: Vec<RightKey>, // unique -> public, format: <>.<>.<>
    default_values: Vec<bool>,  // true or false
}

impl Rights {
    pub fn new(
        rights_ids: Vec<RightId>,
        rights_keys: Vec<RightKey>,
        default_values: Vec<bool>,
    ) -> Self {
        Self {
            rights_ids,
            rights_keys,
            default_values,
        }
    }

    pub fn get_right(&self, key: &RightKey) -> AppResult<RightId> {
        let index = self.get_index(key)?;
        Ok(self.rights_ids[index])
    }

    fn get_index(&self, right_key: &RightKey) -> AppResult<usize> {
        self.rights_keys
            .iter()
            .position(|i| i == right_key)
            .ok_or(AppError::Internal(
                "Cannot get index with RightKey does not exist".to_string(),
            ))
    }
}

pub struct Role {
    name: String,
    rights: HashMap<RightId, bool>,
}

impl Role {
    pub fn new(name: &str, rights_ids: &[RightId], default_value: &[bool]) -> Self {
        let mut rights = HashMap::with_capacity(rights_ids.len());
        let pairs = rights_ids
            .iter()
            .cloned()
            .zip(default_value.iter().cloned());
        rights.extend(pairs);

        Self {
            name: name.to_string(),
            rights,
        }
    }
    pub fn add_right(&mut self, right_id: RightId) {
        self.rights.insert(right_id, true);
    }

    pub fn remove_right(&mut self, right_id: RightId) {
        self.rights.insert(right_id, false);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rights(&self) -> &HashMap<RightId, bool> {
        &self.rights
    }
}

impl RoleService {
    pub fn new(rights: Rights) -> Self {
        Self {
            rights,
            roles_ids: Vec::new(),
            roles: Vec::new(),
        }
    }

    pub fn create_role(&mut self, name: &str) -> AppResult<()> {
        let role = Role::new(name, &self.rights.rights_ids, &self.rights.default_values);
        self.increment_roles_ids();
        self.roles.push(role);
        Ok(())
    }

    pub fn get_mut_role(&mut self, rid: &RoleId) -> AppResult<&mut Role> {
        let index = self.get_index(rid)?;
        self.roles
            .get_mut(index)
            .ok_or(AppError::NotFound("no such role".to_string()))
    }
    pub fn get_role(&self, rid: &RoleId) -> AppResult<&Role> {
        let index = self.get_index(rid)?;
        self.roles
            .get(index)
            .ok_or(AppError::NotFound("no such role".to_string()))
    }

    pub fn remove_role(&mut self, rid: &RoleId) -> AppResult<()> {
        let index = self.get_index(rid)?;

        self.roles.remove(index);
        self.roles_ids.remove(index);
        Ok(())
    }

    pub fn is_available_action(
        &self,
        user_role_id: &RoleId,
        right_key: &RightKey,
    ) -> AppResult<bool> {
        let role = self.get_role(user_role_id)?;
        let right_id = self.rights.get_right(right_key)?;
        let roles_rights = role.rights();

        Ok(roles_rights[&right_id])
    }

    fn get_index(&self, rid: &RoleId) -> AppResult<usize> {
        self.roles_ids
            .iter()
            .position(|i| i == rid)
            .ok_or(AppError::Internal(
                "Cannot get index with RoleID does not exist".to_string(),
            ))
    }
    fn increment_roles_ids(&mut self) {
        let role_id = self.roles_ids.last().unwrap_or(&RoleId(0)).increment();
        self.roles_ids.push(role_id);
    }
}
