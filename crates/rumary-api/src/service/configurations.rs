use crate::error::{AppError, AppResult};
use crate::repo::repository::ConfigurationRepository;
use rumary_dto::domain::api::{Configuration, NewConfiguration, UpdateConfiguration};
use rumary_dto::dto::api::request::{ConfigurationsRequest, GetConfigurationRequest, NewConfigurationRequest, UpdateConfigurationRequest};
use std::sync::Arc;

pub struct ConfigurationService {
    configuration_repo: Arc<dyn ConfigurationRepository<Error=AppError>>,
}

impl ConfigurationService {
    pub(crate) fn new(configuration_repo: Arc<dyn ConfigurationRepository<Error=AppError>>) -> Self {
        Self { configuration_repo }
    }

    pub async fn create_configuration(&self, request: NewConfigurationRequest) -> AppResult<Configuration> {
        let new_configuration = NewConfiguration {
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            instance_uuid: request.instance_uuid,
        };

        self.configuration_repo.create_config(new_configuration).await
    }

    pub async fn update_configuration(&self, request: UpdateConfigurationRequest) -> AppResult<Configuration> {
        let update_configuration = UpdateConfiguration {
            uuid: request.uuid,
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            instance_uuid: request.instance_uuid,
        };

        self.configuration_repo.update_config(update_configuration).await
    }

    pub async fn get_config(&self, request: GetConfigurationRequest, access_level: u16) -> AppResult<Configuration> {
        self.configuration_repo.get_config(request.configuration_uuid, access_level).await
    }

    pub async fn list_configs(&self, request: ConfigurationsRequest, access_level: u16) -> AppResult<Vec<Configuration>> {
        self.configuration_repo.list_configs(request.instance_uuid, access_level).await
    }
}