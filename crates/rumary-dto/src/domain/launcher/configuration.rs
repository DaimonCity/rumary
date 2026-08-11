use crate::domain::launcher::value_object::download_url::DownloadUrl;
use crate::domain::launcher::CheckDirs;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct Configuration {
    pub uuid: Uuid,
    pub name: String,
    pub icon: DownloadUrl,
    pub hard_check: CheckDirs,
    pub soft_check: CheckDirs,
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
            uuid: id,
            name,
            icon,
            hard_check,
            soft_check,
        }
    }
}
