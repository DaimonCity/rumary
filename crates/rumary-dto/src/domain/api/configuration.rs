use crate::domain::api::RightKey;
use crate::domain::configuration::ConfigurationId;
use crate::domain::instance::InstanceId;
use crate::domain::name::{DirectoryName, DisplayName};
use crate::domain::url::IconUrl;

pub struct NewConfiguration {
    pub icon: IconUrl,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub instance_id: InstanceId,
}

pub struct UpdateConfiguration {
    pub icon: Option<IconUrl>,
    pub dir_name: Option<DirectoryName>,
    pub display_name: Option<DisplayName>,
    pub instance_id: Option<InstanceId>,
}

pub struct Configuration {
    pub id: ConfigurationId,
    pub icon: IconUrl,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub instance_id: InstanceId,
}

impl Configuration {
    pub fn default_rights_setup(&self) -> Vec<(RightKey<'static>, bool)> {
        let id_str = self.id.to_string();
        vec![
            (RightKey::get_configuration_key(&id_str), false),
            (RightKey::update_configuration_key(&id_str), false),
            (RightKey::delete_configuration_key(&id_str), false),
        ]
    }
}