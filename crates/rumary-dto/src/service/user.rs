use crate::domain::api::User;
use crate::dto::api::response::ProfileResponse;

impl User {
    pub fn to_profile_response(&self, has_totp: bool) -> ProfileResponse {
        ProfileResponse {
            login: String::from(self.login.as_str()),
            nickname: String::from(self.nickname.as_str()),
            has_totp,
        }
    }
}