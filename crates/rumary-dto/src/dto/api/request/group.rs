use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewGroupRequest {
    pub name: String,
    pub weight: i32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateGroupRequest {
    pub allow_keys: Vec<String>,
    pub remove_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupWeightRequest {
    pub weight: i32,
}

#[derive(Debug, Deserialize)]
pub struct PermissionGrantDto {
    pub key: String,
    pub allow: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupPermissionsRequest {
    #[serde(default)]
    pub grant: Vec<PermissionGrantDto>,
    #[serde(default)]
    pub revoke: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct AddGroupParentRequest {
    pub parent: String,
}
