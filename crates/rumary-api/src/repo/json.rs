use crate::error::AppError;
use crate::repo::repository::RightsRepository;
use crate::service::right::Rights;
use async_trait::async_trait;
use rumary_dto::domain::api::{RightDefinition, RightFromRow, RightId, RightKey};
use std::path::PathBuf;
use tokio::fs;

const DEFAULT_PATH: &str = "rights.json";

pub struct LocalRightsRepo {
    path: PathBuf,
    channel: tokio::sync::mpsc::Sender<Rights>,
}

impl LocalRightsRepo {
    pub fn new(path: Option<PathBuf>, channel: tokio::sync::mpsc::Sender<Rights>) -> Self {
        if let Some(item) = path {
            Self {
                path: item,
                channel,
            }
        } else {
            Self {
                path: DEFAULT_PATH.into(),
                channel,
            }
        }
    }

    async fn read_rights_or_empty(&self) -> Result<RightFromRow, AppError> {
        let content = fs::read_to_string(self.path.clone())
            .await
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).map_err(AppError::JsonError)
    }

    async fn write_and_publish(&self, rights: RightFromRow) -> Result<(), AppError> {
        let json_data = serde_json::to_string_pretty(&rights)?;
        fs::write(self.path.clone(), json_data).await?;
        self.channel.send(rights.into()).await?;
        Ok(())
    }

    fn next_right_id(rights: &RightFromRow) -> RightId {
        rights
            .values()
            .map(|definition| definition.id)
            .max()
            .unwrap_or_else(RightId::start)
            .increment()
    }
}

#[async_trait]
impl RightsRepository for LocalRightsRepo {
    type Error = AppError;

    async fn add_right(&self, right_key: RightKey, default_value: bool) -> Result<(), Self::Error> {
        let mut rights = self.read_rights_or_empty().await?;
        let key = String::from(right_key);

        if rights.contains_key(&key) {
            return Err(AppError::Conflict(format!("right {key} already exists")));
        }

        let right_id = Self::next_right_id(&rights);
        rights.insert(key, RightDefinition::new(right_id, default_value));

        self.write_and_publish(rights).await
    }

    async fn update_right(
        &self,
        right_key: RightKey,
        default_value: bool,
    ) -> Result<(), Self::Error> {
        let mut rights = self.read_rights_or_empty().await?;
        let key = String::from(right_key);

        let definition = rights
            .get_mut(&key)
            .ok_or_else(|| AppError::NotFound(format!("right {key} does not exist")))?;
        definition.default = default_value;

        self.write_and_publish(rights).await
    }

    async fn get_rights(&self) -> Result<Rights, Self::Error> {
        let content = fs::read_to_string(self.path.clone()).await?;
        let rights: RightFromRow = serde_json::from_str(&content)?;
        Ok(rights.into())
    }

    async fn remove_right(&self, right_key: RightKey) -> Result<(), Self::Error> {
        let mut rights = self.read_rights_or_empty().await?;
        let key = String::from(right_key);

        let definition = rights
            .get_mut(&key)
            .ok_or_else(|| AppError::NotFound(format!("right {key} does not exist")))?;
        definition.active = false;

        self.write_and_publish(rights).await
    }
}
