use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InstancePathRequest {
    pub path: PathBuf
}