use crate::dto::api::db::role::RoleFromRow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

type RoleResult<T> = Result<T, RoleError>;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct RightId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RoleId(usize);

#[derive(Debug)]
pub enum RoleError {
    NotFound(String),
    Exists(String),
}

impl From<RoleId> for usize {
    fn from(id: RoleId) -> Self {
        id.0
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Ord, PartialOrd)]
pub struct RightKey(String);

impl RightKey {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<RightKey> for String {
    fn from(value: RightKey) -> Self {
        value.0
    }
}

impl Display for RightKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RightId {
    pub fn start() -> Self {
        Self(0)
    }
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

impl RoleId {
    pub fn start() -> Self {
        Self(0)
    }
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RightDefinition {
    pub id: RightId,
    pub default: bool,
    pub active: bool,
}

impl RightDefinition {
    pub fn new(id: RightId, default: bool) -> Self {
        Self {
            id,
            default,
            active: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RightFromRow(pub HashMap<String, RightDefinition>);

#[derive(Clone, Debug)]
pub struct RoleRights(pub(crate) HashMap<RightId, bool>);
impl RoleRights {
    pub fn new(c: HashMap<RightId, bool>) -> Self {
        Self(c)
    }

    pub fn contains(&self, id: &RightId) -> bool {
        self.0.contains_key(id)
    }
    pub fn get(&self, id: &RightId) -> Option<bool> {
        self.0.get(id).copied()
    }
    pub fn insert(&mut self, id: RightId, value: bool) {
        self.0.insert(id, value);
    }
    pub fn remove(&mut self, id: &RightId) -> bool {
        self.0.remove(id).is_some()
    }
}

impl Deref for RightFromRow {
    type Target = HashMap<String, RightDefinition>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RightFromRow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RoleRights {
    type Target = HashMap<RightId, bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RoleRights {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    name: String,
    rights: RoleRights,
}

pub struct NewRole {
    rid: RoleId,
    name: String,
    rights: RoleRights,
}

pub struct UpdateRoleDb {
    rid: RoleId,
    name: String,
    rights: RoleRights,
}

pub struct UpdateRole {
    pub allow_keys: Vec<RightKey>,
    pub remove_keys: Vec<RightKey>,
}

impl NewRole {
    pub fn new(rid: RoleId, role: Role) -> Self {
        Self {
            rid,
            name: role.name,
            rights: role.rights,
        }
    }
}

impl UpdateRole {
    pub fn new(allow_keys: Vec<RightKey>,
               remove_keys: Vec<RightKey>,) -> Self {
        Self {
            allow_keys,
            remove_keys
        }
    }
}

impl UpdateRoleDb {
    pub fn new(rid: RoleId, role: Role) -> Self {
        Self {
            rid,
            name: role.name,
            rights: role.rights,
        }
    }
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
            rights: RoleRights(rights),
        }
    }
    pub fn add_right(&mut self, right_id: &RightId) {
        self.rights.insert(*right_id, true);
    }

    pub fn add_rights(&mut self, right_ids: &[RightId]) {
        let _ = right_ids.iter().map(|right_id| self.add_right(right_id));
    }

    pub fn remove_right(&mut self, right_id: &RightId) {
        self.rights.insert(*right_id, false);
    }

    pub fn remove_rights(&mut self, right_ids: &[RightId]) {
        let _ = right_ids.iter().map(|right_id| self.remove_right(right_id));
    }
    pub fn from(role_dto: RoleFromRow) -> Self {
        Self {
            name: role_dto.name,
            rights: RoleRights(role_dto.rights),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rights(&self) -> &HashMap<RightId, bool> {
        &self.rights
    }
    pub fn rights_cloned(&self) -> RoleRights {
        self.rights.clone()
    }
    pub fn allow_all(&mut self) {
        self.rights.iter_mut().for_each(|(_, v)| *v = true);
    }
    pub fn deny_all(&mut self) {
        self.rights.iter_mut().for_each(|(_, v)| *v = false);
    }

    pub fn set_rights(&mut self, rights: HashMap<RightId, bool>) {
        self.rights = RoleRights(rights);
    }

    pub fn reconcile_rights(&mut self, rights_ids: &[RightId], default_values: &[bool]) {
        let defaults = rights_ids
            .iter()
            .copied()
            .zip(default_values.iter().copied());
        let mut next = HashMap::with_capacity(rights_ids.len());

        for (right_id, default_value) in defaults {
            let value = self.rights.get(&right_id).unwrap_or(default_value);
            next.insert(right_id, value);
        }

        self.rights = RoleRights(next);
    }
}
