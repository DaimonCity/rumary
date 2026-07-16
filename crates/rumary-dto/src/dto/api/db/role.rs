use crate::domain::api::{RightId, Role, RoleId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RoleFromRow {
    pub id: RoleId,
    pub name: String,
    pub rights: HashMap<RightId, bool>,
}

impl From<RoleFromRow> for Role {
    fn from(role: RoleFromRow) -> Self {
        Role::from(role)
    }
}
