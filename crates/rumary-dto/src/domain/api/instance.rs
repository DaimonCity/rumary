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
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub loader: Option<Loader>
}

// need to change
pub struct Instance {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: Loader
}
