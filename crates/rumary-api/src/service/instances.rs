use crate::error::{AppError, AppResult};
use crate::repo::repository::InstanceRepository;
use rumary_dto::domain::api::value_object::instance::InstanceId;
use rumary_dto::domain::api::{Instance, NewInstance, UpdateInstance};
use std::sync::Arc;

pub struct InstanceService {
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
}

impl InstanceService {
    pub(crate) fn new(instance_repo: Arc<dyn InstanceRepository<Error = AppError>>) -> Self {
        Self { instance_repo }
    }

    pub async fn create(&self, new_instance: NewInstance) -> AppResult<Instance> {
        self.instance_repo.create_instance(new_instance).await
    }

    pub async fn update(
        &self,
        instance_id: InstanceId,
        update_instance: UpdateInstance,
    ) -> AppResult<Instance> {
        self.instance_repo
            .update_instance(instance_id, update_instance)
            .await
    }
    pub async fn get(
        &self,
        instance_id: InstanceId,
    ) -> AppResult<Instance> {
        let instance = self
            .instance_repo
            .get_instance(instance_id)
            .await?;
        Ok(instance)
    }

    pub async fn list(&self) -> AppResult<Vec<Instance>> {
        let instances = self.instance_repo.list_instances().await?;
        Ok(instances)
    }

    pub async fn delete(&self, instance_id: InstanceId) -> AppResult<Instance> {
        self.instance_repo.delete_instance(instance_id).await
    }
}
