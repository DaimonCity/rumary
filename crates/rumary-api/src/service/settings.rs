use std::path::PathBuf;
use std::sync::Arc;
use crate::error::AppError;
use crate::repo::repository::SettingsRepository;

pub struct SettingsService {
    settings_repo: Arc<dyn SettingsRepository<Error=AppError>>,
    configuration_dir_path: PathBuf,
}

