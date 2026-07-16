use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewRoleRequest {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRoleRequest {
    pub allow_keys: Vec<String>,
    pub remove_keys: Vec<String>,
}
