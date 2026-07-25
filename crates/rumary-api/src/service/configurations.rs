use crate::error::{AppError, AppResult};
use crate::repo::repository::ConfigurationRepository;
use rumary_dto::domain::api::Configuration;
use rumary_dto::domain::configuration::ConfigurationId;
use rumary_dto::dto::api::request::{
    NewConfigurationRequest,
    UpdateConfigurationRequest,
};
use rumary_dto::dto::api::response::GetConfigurationResponse;
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

    pub async fn create_configuration(
        &self,
        request: NewConfigurationRequest,
    ) -> AppResult<Configuration> {
        self.configuration_repo
            .create_config(request.try_into()?)
            .await
    }

    pub async fn update_configuration(
        &self,
        configuration_id: ConfigurationId,
        request: UpdateConfigurationRequest,
    ) -> AppResult<Configuration> {
        self.configuration_repo
            .update_config(configuration_id, request.try_into()?)
            .await
    }

    pub async fn get_config(
        &self,
        config_id: ConfigurationId,
    ) -> AppResult<GetConfigurationResponse> {
        let _config = self.configuration_repo.get_config(config_id).await?;
        todo!()
    }

    /// configuration.method.list
    pub async fn list_configs(&self, available_ids: &[ConfigurationId]) -> AppResult<Vec<GetConfigurationResponse>> {
        let instances = self.configuration_repo.list_all_configs(available_ids).await?;
        Ok(instances.into_iter().map(Into::into).collect())
    }
    
    pub async fn delete_configuration(&self, configuration_id: ConfigurationId) -> AppResult<Configuration> {
        self.configuration_repo.delete_config(configuration_id).await
    }
}
