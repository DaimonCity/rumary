use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub login: String,
    pub nickname: String,
    pub has_totp: bool,
}

#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub permissions: Vec<String>,
}
