use crate::domain::api::loader::Loader;
use crate::domain::api::RightKey;
use crate::domain::instance::InstanceId;
use crate::domain::name::{Description, DirectoryName, DisplayName};
use crate::domain::url::IconUrl;
use crate::domain::value_object::version::Version;

pub struct NewInstance {
    pub icon: IconUrl,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub version: Version,
    pub description: Description,
    pub loader: Loader,
}

pub struct UpdateInstance {
    pub icon: Option<IconUrl>,
    pub dir_name: Option<DirectoryName>,
    pub display_name: Option<DisplayName>,
    pub version: Option<Version>,
    pub description: Option<Description>,
    pub loader: Option<Loader>,
}

pub struct Instance {
    pub id: InstanceId,
    pub icon: IconUrl,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub version: Version,
    pub description: Description,
    pub loader: Loader,
}

impl Instance {
    /// Возвращает базовые права, связанные с этим инстансом, и их статус по умолчанию
    pub fn default_rights_setup(id: &InstanceId) -> Vec<(RightKey<'static>, bool)> {
        vec![
            (RightKey::get_instance_key(id), false),
            (RightKey::update_instance_key(id), false),
            (RightKey::delete_instance_key(id), false),
        ]
    }
}