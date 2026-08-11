use crate::domain::{api, perms};

impl From<api::value_object::user::UserId> for perms::value_object::user::UserId {
    fn from(value: api::value_object::user::UserId) -> Self {
        Self(value.0)
    }
}

impl From<perms::value_object::user::UserId> for api::value_object::user::UserId {
    fn from(value: perms::value_object::user::UserId) -> Self {
        Self(value.0)
    }
}
