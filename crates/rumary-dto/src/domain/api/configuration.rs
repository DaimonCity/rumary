use crate::domain::api::value_object::configuration::ConfigurationId;
use crate::domain::api::value_object::instance::InstanceId;
use crate::domain::api::value_object::name::{DirectoryName, DisplayName};
use crate::domain::api::value_object::url::IconUrl;

pub struct NewConfiguration {
    pub icon: IconUrl,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub instance_id: InstanceId,
    pub is_public: bool,
}

pub struct UpdateConfiguration {
    pub icon: Option<IconUrl>,
    pub dir_name: Option<DirectoryName>,
    pub display_name: Option<DisplayName>,
    pub instance_id: Option<InstanceId>,
}

pub struct Configuration {
    pub id: ConfigurationId,
    pub icon: Option<IconUrl>,
    pub dir_name: DirectoryName,
    pub display_name: DisplayName,
    pub instance_id: InstanceId,
    pub is_public: bool,
}
