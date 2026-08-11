use std::collections::BTreeMap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct GroupPermissionResponse {
    pub key: String,               // PermissionKey::as_str().to_string()
    pub allow: bool,
    pub context: BTreeMap<String, String>, // из ContextSet
    pub source_priority: i32,      // SourcePriority::get() или как там метод называется
    pub expires_at: Option<DateTime<Utc>>, // NodeExpiry::get()
}

#[derive(Debug, Serialize)]
pub struct GroupSummaryResponse {
    pub name: String,
    pub weight: i32,
}

#[derive(Debug, Serialize)]
pub struct GetGroupResponse {
    pub name: String,
    pub weight: i32,
    pub permissions: Vec<GroupPermissionResponse>,
    pub members: Vec<Uuid>,
    pub parents: Vec<String>,
}