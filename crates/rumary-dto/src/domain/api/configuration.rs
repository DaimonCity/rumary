use uuid::Uuid;

pub struct NewConfiguration {
    pub display_name: String,
    pub client_uuid: String,
    pub dir_name: String,
    pub icon: String,
}

pub struct UpdateConfiguration {
    pub display_name: String,
    pub client_uuid: String,
    pub dir_name: String,
    pub icon: String,
}

pub struct Configuration {
    pub uuid: Uuid,
    pub display_name: String,
    pub instance_uuid: Uuid,
    pub dir_name: String,
    pub icon: String,
}