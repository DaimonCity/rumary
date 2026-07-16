use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GetRoleResponse {
    pub id: usize,
    pub name: String,
    pub rights: RoleRightsResponse
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleRightsResponse(pub(crate) HashMap<usize, bool>);