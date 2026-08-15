use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub otp_auth_url: String,
}
