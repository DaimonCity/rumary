use crate::error::{AppError, AppResult};
use crate::repo::repository::InstanceRepository;
use rumary_dto::domain::api::{Instance, Loader, NewInstance, UpdateInstance};
use rumary_dto::dto::api::request::{GetInstanceRequest, NewInstanceRequest, UpdateInstanceRequest};
use std::sync::Arc;

pub struct InstanceService {
    instance_repo: Arc<dyn InstanceRepository<Error=AppError>>,
}

impl InstanceService {
    pub(crate) fn new(instance_repo: Arc<dyn InstanceRepository<Error=AppError>>) -> Self {
        Self {instance_repo }
    }

    pub async fn create_instance(&self, request: NewInstanceRequest) -> AppResult<Instance> {
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

    pub async fn update_instance(&self, request: UpdateInstanceRequest) -> AppResult<Instance> {
        let loader = match request.loader {
            Some(ref l) if l != "vanilla" && request.loader_version.is_none() => {
                return Err(AppError::Internal("Missing loader_version field".into()));
            }
            Some(l) => Some(Loader::from_strings(l, request.loader_version)),
            None => None,
        };


        let new_instance = UpdateInstance {
            uuid: request.uuid,
            icon: request.icon,
            dir_name: request.dir_name,
            display_name: request.display_name,
            version: request.version,
            description: request.description,
            loader
        };

        self.instance_repo.update_instance(new_instance).await
    }
    
    pub async fn get_instance(&self, request: GetInstanceRequest, access_level: u16) -> AppResult<Instance> {
        self.instance_repo.get_instance(request.instance_uuid, access_level).await
    }
    
    pub async fn list_instances(&self, access_level: u16) -> AppResult<Vec<Instance>> {
        self.instance_repo.list_instances(access_level).await
    }
}