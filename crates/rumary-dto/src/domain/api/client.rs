use uuid::Uuid;

#[allow(dead_code)]
pub struct Client {
    id: Uuid,
    display_name: String,
    icon: String,
    minecraft_version: String,
    url: String,
    loader: String,
    loader_version: String,
}
