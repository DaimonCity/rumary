use crate::domain::api::{RightId, RightKey, RoleRights, UpdateRole};
use crate::dto::api::request::UpdateRoleRequest;
use crate::dto::api::response::role::RoleRightsResponse;
use std::collections::HashMap;

impl<'a> From<String> for RightKey<'a> {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<UpdateRoleRequest> for UpdateRole<'_> {
    fn from(value: UpdateRoleRequest) -> Self {
        Self::new(
            value.allow_keys.into_iter().map(Into::into).collect(),
            value.remove_keys.into_iter().map(Into::into).collect(),
        )
    }
}



impl From<RoleRights> for RoleRightsResponse {
    fn from(value: RoleRights) -> Self {
        let hashmap: HashMap<usize, _> = value.0.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self(hashmap)
    }
}
impl From<RightId> for usize {
    fn from(value: RightId) -> Self {
        value.value()
    }
}
