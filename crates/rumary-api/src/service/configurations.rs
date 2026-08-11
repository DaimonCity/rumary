use crate::error::{AppError, AppResult};
use crate::repo::repository::ConfigurationRepository;
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::domain::api::value_object::instance::InstanceId;
use rumary_dto::domain::api::{Configuration, NewConfiguration, UpdateConfiguration};
use std::sync::Arc;

pub struct ConfigurationService {
    configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
}

impl ConfigurationService {
    pub(crate) fn new(
        configuration_repo: Arc<dyn ConfigurationRepository<Error = AppError>>,
    ) -> Self {
        Self { configuration_repo }
    }

    pub async fn create(&self, new_config: NewConfiguration) -> AppResult<Configuration> {
        self.configuration_repo
            .create_config(new_config)
            .await
    }

    pub async fn update(
        &self,
        configuration_id: ConfigurationId,
        update_config: UpdateConfiguration,
    ) -> AppResult<Configuration> {
        self.configuration_repo
            .update_config(configuration_id, update_config)
            .await
    }

    pub async fn get(&self, config_id: ConfigurationId) -> AppResult<Configuration> {
        self.configuration_repo.get_config(config_id).await
    }

    pub async fn list(&self) -> AppResult<Vec<Configuration>> {
        let instances = self.configuration_repo.list_all_configs().await?;
        Ok(instances)
    }
    pub async fn list_for_instance(&self, instance_id: InstanceId) -> AppResult<Vec<Configuration>> {
        let instances = self.configuration_repo.list_for_instance(instance_id).await?;
        Ok(instances)
    }

    pub async fn delete(
        &self,
        configuration_id: ConfigurationId,
    ) -> AppResult<Configuration> {
        self.configuration_repo
            .delete_config(configuration_id)
            .await
    }
}
