use crate::domain::api::{Configuration, NewConfiguration, UpdateConfiguration};
use crate::domain::api::value_object::error::ValueObjectError;
use crate::domain::api::value_object::instance::InstanceId;
use crate::domain::api::value_object::name::{DirectoryName, DisplayName};
use crate::domain::api::value_object::url::IconUrl;
use crate::dto::api::request::{NewConfigurationRequest, UpdateConfigurationRequest};
use crate::dto::api::response::GetConfigurationResponse;

impl From<Configuration> for GetConfigurationResponse {
    fn from(config: Configuration) -> Self {
        Self {
            id: config.id.into(),
            icon: config.icon.map(Into::into),
            dir_name: config.dir_name.into(),
            display_name: config.display_name.into(),
            instance_id: config.instance_id.into(),
            hard_dirs: vec![],         // ?
            soft_dirs: vec![],         // ?
            files: Default::default(), // ?
        }
    }
}

impl TryFrom<NewConfigurationRequest> for NewConfiguration {
    type Error = ValueObjectError;

    fn try_from(request: NewConfigurationRequest) -> Result<Self, Self::Error> {
        let dir_name = request.dir_name.try_into()?;
        let display_name = request.display_name.try_into()?;
        let icon = request.icon.try_into()?;
        Ok(NewConfiguration {
            icon,
            dir_name,
            display_name,
            instance_id: request.instance_id.into(),
            is_public: request.is_public,
        })
    }
}

impl TryFrom<UpdateConfigurationRequest> for UpdateConfiguration {
    type Error = ValueObjectError;

    fn try_from(value: UpdateConfigurationRequest) -> Result<Self, Self::Error> {
        let instance_id = value.instance_id.map(InstanceId::from);
        let dir_name = value.dir_name.map(DirectoryName::try_from).transpose()?;
        let display_name = value.display_name.map(DisplayName::try_from).transpose()?;
        let icon = value.icon.map(IconUrl::try_from).transpose()?;
        Ok(UpdateConfiguration {
            icon,
            dir_name,
            display_name,
            instance_id,
        })
    }
}
