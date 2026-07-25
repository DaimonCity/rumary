use crate::error::{AppError, AppResult};
use crate::repo::repository::InstanceRepository;
use rumary_dto::domain::api::{Instance, Loader, NewInstance, UpdateInstance};
use rumary_dto::domain::name::{Description, DirectoryName, DisplayName};
use rumary_dto::domain::url::IconUrl;
use rumary_dto::domain::version::Version;
use rumary_dto::dto::api::request::{
    GetInstanceRequest, NewInstanceRequest, UpdateInstanceResponse,
};
use rumary_dto::dto::api::response::GetInstanceResponse;
use std::sync::Arc;
use rumary_dto::domain::instance::InstanceId;

pub struct InstanceService {
    instance_repo: Arc<dyn InstanceRepository<Error = AppError>>,
}

impl InstanceService {
    pub(crate) fn new(instance_repo: Arc<dyn InstanceRepository<Error = AppError>>) -> Self {
        Self { instance_repo }
    }

    pub async fn create_instance(&self, request: NewInstanceRequest) -> AppResult<Instance> {
        let version = request.version.try_into()?;
        let loader_version = request.loader_version.map(Version::try_from).transpose()?;
        let new_instance = NewInstance {
            icon: request.icon.try_into()?,
            dir_name: request.dir_name.try_into()?,
            display_name: request.display_name.try_into()?,
            version,
            description: request.description.try_into()?,
            loader: Loader::from_string(request.loader, loader_version)?,
        };

        self.instance_repo.create_instance(new_instance).await
    }

    pub async fn update_instance(&self, request: UpdateInstanceResponse) -> AppResult<Instance> {
        let loader = if let Some(loader) = request.loader {
            let loader_version = request.loader_version.map(Version::try_from).transpose()?;
            Some(Loader::from_string(loader, loader_version)?)
        } else {
            None
        };

        let icon = request.icon.map(IconUrl::try_from).transpose()?;
        let dir_name = request.dir_name.map(DirectoryName::try_from).transpose()?;
        let display_name = request
            .display_name
            .map(DisplayName::try_from)
            .transpose()?;
        let version = request.version.map(Version::try_from).transpose()?;
        let description = request.description.map(Description::try_from).transpose()?;

        let update_instance = UpdateInstance {
            icon,
            dir_name,
            display_name,
            version,
            description,
            loader,
        };

        self.instance_repo.update_instance(update_instance).await
    }
    /// instance.<instance-uuid>.get
    pub async fn get_instance(
        &self,
        request: GetInstanceRequest,
    ) -> AppResult<GetInstanceResponse> {
        let instance = self
            .instance_repo
            .get_instance(request.instance_id.into())
            .await?;
        Ok(instance.into())
    }

    /// instance.method.list
    pub async fn list_instances(&self, available_ids: &[InstanceId]) -> AppResult<Vec<GetInstanceResponse>> {
        let instances = self.instance_repo.list_instances(available_ids).await?;
        Ok(instances.into_iter().map(Into::into).collect())
    }

    pub async fn delete_instance(&self, instance_id: InstanceId) -> AppResult<Instance> {
        self.instance_repo.delete_instance(instance_id).await
    }
}
