use crate::domain::api::value_object::error::ValueObjectError;
use crate::domain::api::value_object::name::{Description, DirectoryName, DisplayName};
use crate::domain::api::value_object::url::IconUrl;
use crate::domain::api::value_object::version::Version;
use crate::domain::api::{Instance, Loader, NewInstance, UpdateInstance};
use crate::dto::api::request::{NewInstanceRequest, UpdateInstanceRequest};
use crate::dto::api::response::GetInstanceResponse;

impl From<Instance> for GetInstanceResponse {
    fn from(value: Instance) -> Self {
        let loader_version = value.loader.get_version().map(String::from);
        Self {
            id: value.id.into(),
            icon: value.icon.into(),
            dir_name: value.dir_name.into(),
            display_name: value.display_name.into(),
            version: value.version.into(),
            description: value.description.into(),
            loader: value.loader.into(),
            loader_version,
        }
    }
}

impl TryFrom<NewInstanceRequest> for NewInstance {
    type Error = ValueObjectError;

    fn try_from(value: NewInstanceRequest) -> Result<Self, Self::Error> {
        let version = value.version.try_into()?;
        let loader_version = value.loader_version.map(Version::try_from).transpose()?;
        Ok(NewInstance {
            icon: value.icon.try_into()?,
            dir_name: value.dir_name.try_into()?,
            display_name: value.display_name.try_into()?,
            version,
            description: value.description.try_into()?,
            loader: Loader::from_string(value.loader, loader_version)?,
            is_public: value.is_public,
        })
    }
}

impl TryFrom<UpdateInstanceRequest> for UpdateInstance {
    type Error = ValueObjectError;

    fn try_from(value: UpdateInstanceRequest) -> Result<Self, Self::Error> {
        let loader = if let Some(loader) = value.loader {
            let loader_version = value.loader_version.map(Version::try_from).transpose()?;
            Some(Loader::from_string(loader, loader_version)?)
        } else {
            None
        };

        let icon = value.icon.map(IconUrl::try_from).transpose()?;
        let dir_name = value.dir_name.map(DirectoryName::try_from).transpose()?;
        let display_name = value.display_name.map(DisplayName::try_from).transpose()?;
        let version = value.version.map(Version::try_from).transpose()?;
        let description = value.description.map(Description::try_from).transpose()?;

        Ok(UpdateInstance {
            icon,
            dir_name,
            display_name,
            version,
            description,
            loader,
        })
    }
}
