use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub otp_auth_url: String,
}
