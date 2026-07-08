use uuid::Uuid;
use crate::domain::api::RoleType;

pub struct Role {
    pub uuid: Uuid,
    pub name: String,
    pub role_type: RoleType,
    pub level: u16
}

pub struct NewRole {
    pub name: String,
    pub role_type: RoleType,
    pub level: u16
}