use crate::domain::api::{AccessLevel, RoleType};
use crate::dto::api::request::{AccessLevelRequest, RoleTypeRequest};
use crate::dto::api::response::{AccessLevelResponse, RoleTypeResponse};

impl From<AccessLevelRequest> for AccessLevel {
    fn from(request: AccessLevelRequest) -> Self {
        Self {
            role_type: request.role_type.into(),
            level: request.level,
        }
    }
}

impl From<AccessLevel> for AccessLevelResponse {
    fn from(value: AccessLevel) -> Self {
        Self {
            role_type: value.role_type.into(),
            level: value.level,
        }
    }
}
impl From<RoleTypeRequest> for RoleType {
    fn from(request: RoleTypeRequest) -> Self {
        match request {
            RoleTypeRequest::User => Self::User,
            RoleTypeRequest::VipUser => Self::VipUser,
            RoleTypeRequest::Worker => Self::Worker,
            RoleTypeRequest::Owner => Self::Owner,
        }
    }
}

impl From<RoleType> for RoleTypeResponse {
    fn from(request: RoleType) -> Self {
        match request {
            RoleType::User => Self::User,
            RoleType::VipUser => Self::VipUser,
            RoleType::Worker => Self::Worker,
            RoleType::Owner => Self::Owner,
        }
    }
}