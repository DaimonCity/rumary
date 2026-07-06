use uuid::Uuid;
use crate::domain::api::loader::Loader;

pub struct NewInstance {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: Loader
}

pub struct UpdateInstance {
    pub uuid: Uuid,
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub loader: Option<Loader>
}

pub struct DeleteInstance {
    pub uuid: Uuid,
}

pub struct Instance {
    pub uuid: Uuid,
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: Loader
}
