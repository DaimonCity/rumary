use crate::domain::launcher::{CheckDirs};
use std::fmt::Debug;
use uuid::Uuid;
use crate::domain::download_url::DownloadUrl;

#[derive(Debug)]
pub struct Configuration {
    id: Uuid,
    name: String,
    icon: DownloadUrl,
    hard_check: CheckDirs,
    soft_check: CheckDirs,
}

impl Configuration {
    pub fn new(
        id: Uuid,
        name: String,
        icon: DownloadUrl,
        hard_check: CheckDirs,
        soft_check: CheckDirs,
    ) -> Self {
        Self {
            id,
            name,
            icon,
            hard_check,
            soft_check,
        }
    }
}