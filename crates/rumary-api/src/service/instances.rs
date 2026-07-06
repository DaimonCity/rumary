use crate::error::{AppError, AppResult};
use crate::repo::repository::InstanceRepository;
use rumary_dto::domain::api::{Instance, Loader, NewInstance};
use rumary_dto::dto::api::response::LauncherClientDto;
use std::sync::Arc;
use uuid::Uuid;

pub struct InstanceService {
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
}

impl InstanceService {
    pub(crate) fn new(instance_repo: Arc<dyn InstanceRepository<Error = AppError>>) -> Self {
        Self { instance_repo }
    }

    pub async fn create_instance(&self, request: LauncherClientDto) -> AppResult<Instance> {
        let new_instance = NewInstance {
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            version: request.version,
            description: request.description,
            loader: Loader::from_strings(request.loader, request.loader_version),
        };

        self.instance_repo.create_instance(new_instance).await
    }

    pub async fn get_instance(&self, instance_id: Uuid) -> AppResult<Instance> {
        self.instance_repo.find_instance(instance_id).await
    }

    pub async fn list_instances(&self, access_level: u16) -> AppResult<Vec<Instance>> {
        self.instance_repo.list_instances(access_level).await
    }
}
