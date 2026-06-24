use uuid::Uuid;
use crate::domain::launcher::MinecraftLaunchArgs;

pub struct Client {
    id: Uuid,
    display_name: String,
    icon: String,
    minecraft_version: String,
    url: String,
    loader: String,
    loader_version: String,
    minecraft_launch_args: MinecraftLaunchArgs
}
