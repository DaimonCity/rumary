use crate::domain::api::Instance;
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
