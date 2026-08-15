use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpCodeRequest {
    code: String,
}

impl TotpCodeRequest {
    pub fn code(&self) -> String {
        self.code.clone()
    }
}