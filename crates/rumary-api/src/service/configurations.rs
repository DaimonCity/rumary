use crate::error::{AppError, AppResult};
use crate::repo::repository::ConfigurationRepository;
use rumary_dto::domain::api::Configuration;
use rumary_dto::dto::api::request::{ConfigurationsRequest, GetConfigurationRequest, NewConfigurationRequest, UpdateConfigurationRequest};
use std::sync::Arc;
use rumary_dto::domain::configuration::ConfigurationId;
use rumary_dto::dto::api::response::GetConfigurationResponse;

pub struct ConfigurationService {
    configuration_repo: Arc<dyn ConfigurationRepository<Error=AppError>>,
}

impl ConfigurationService {
    pub(crate) fn new(configuration_repo: Arc<dyn ConfigurationRepository<Error=AppError>>) -> Self {
        Self { configuration_repo }
    }

    pub async fn create_configuration(&self, request: NewConfigurationRequest) -> AppResult<Configuration> {
        self.configuration_repo.create_config(request.try_into()?).await
    }

    pub async fn update_configuration(&self, configuration_id: ConfigurationId, request: UpdateConfigurationRequest) -> AppResult<Configuration> {
        self.configuration_repo.update_config(configuration_id, request.try_into()?).await
    }

    pub async fn get_config(&self, request: GetConfigurationRequest, access_level: u16) -> AppResult<GetConfigurationResponse> {
        let configuration_id = request.configuration_id.into();
        let _config = self.configuration_repo.get_config(configuration_id, access_level).await?;
        todo!()
    }

    pub async fn list_configs(&self, request: ConfigurationsRequest, access_level: u16) -> AppResult<Vec<GetConfigurationResponse>> {
        let instance_id = request.instance_id.into();
        let _configs = self.configuration_repo.list_configs(instance_id, access_level).await?;
        todo!()
    }
}