use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct InstancePathRequest {
    pub path: PathBuf,
}
