use std::sync::Arc;
use rumary_dto::domain::api::{Instance, Loader, NewInstance};
use rumary_dto::dto::api::response::LauncherClientDto;
use crate::error::AppError;
use crate::repository::{InstanceRepo};

pub struct InstanceService {
    instance_repo: Arc<dyn InstanceRepo<Error=AppError>>,
}

impl InstanceService {
    pub(crate) fn new(instance_repo: Arc<dyn InstanceRepo<Error=AppError>>) -> Self {
        Self {instance_repo }
    }

    pub fn create_instance(&self, request: LauncherClientDto) -> Result<Instance, AppError> {
        let new_instance = NewInstance {
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            version: request.version,
            description: request.description,
            loader: Loader::from_strings(request.loader, request.loader_version),
        };

        self.instance_repo.create_instance(new_instance)
    }
}