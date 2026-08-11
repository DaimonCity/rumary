use crate::error::{AppError, AppResult};
use crate::repo::repository::SettingsRepository;
use std::path::Path;
use std::sync::Arc;

pub struct SettingsService {
    settings_repo: Arc<dyn SettingsRepository<Error = AppError>>,
}

impl SettingsService {
    pub fn new(settings_repo: Arc<dyn SettingsRepository<Error = AppError>>) -> Self {
        Self { settings_repo }
    }

    pub async fn set_instance_path(&self, path: &Path) -> AppResult<()> {
        self.settings_repo.save_instance_dir_path(path).await
    }

    pub async fn remove_instance_path(&self) -> AppResult<()> {
        drop(self.settings_repo.delete_instances_dir_path().await?);
        Ok(())
    }
}
