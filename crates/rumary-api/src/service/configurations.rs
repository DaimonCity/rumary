use crate::error::{AppError, AppResult};
use crate::repo::repository::ConfigurationRepository;
use rumary_dto::domain::api::{Configuration, NewConfiguration};
use rumary_dto::dto::api::response::ConfigurationDto;
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
        request: ConfigurationDto,
    ) -> AppResult<Configuration> {
        let new_configuration = NewConfiguration {
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            client_uuid: request.client_uuid,
        };

        self.configuration_repo
            .create_config(new_configuration)
            .await
    }
}
