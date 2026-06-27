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

impl From<AccessLevel> for AccessLevelRequest {
    fn from(request: AccessLevel) -> Self {
        Self {
            role_type: request.role_type.into(),
            level: request.level,
        }
    }
}


macro_rules! impl_from_role {
    ($target:ident) => {
        impl From<RoleType> for $target {
            fn from(request: RoleType) -> Self {
                match request {
                    RoleType::User => Self::User,
                    RoleType::VipUser => Self::VipUser,
                    RoleType::Builder => Self::Builder,
                    RoleType::Writer => Self::Writer,
                    RoleType::Admin => Self::Admin,
                    RoleType::Owner => Self::Owner,
                }
            }
        }
    };
}

impl_from_role!(RoleTypeRequest);
impl_from_role!(RoleTypeResponse);

impl From<RoleTypeRequest> for RoleType {
    fn from(request: RoleTypeRequest) -> Self {
        match request {
            RoleTypeRequest::User => Self::User,
            RoleTypeRequest::VipUser => Self::VipUser,
            RoleTypeRequest::Builder => Self::Builder,
            RoleTypeRequest::Writer => Self::Writer,
            RoleTypeRequest::Admin => Self::Admin,
            RoleTypeRequest::Owner => Self::Owner,
        }
    }
}
