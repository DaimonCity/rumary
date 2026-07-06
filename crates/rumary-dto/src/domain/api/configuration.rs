use uuid::Uuid;

pub struct NewConfiguration {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub instance_uuid: String,
}

pub struct UpdateConfiguration {
    pub uuid: Uuid,
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub instance_uuid: Option<String>,
}

pub struct DeleteConfiguration {
    pub uuid: Uuid,
}

pub struct Configuration {
    pub uuid: Uuid,
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub instance_uuid: String,
}
