use std::collections::HashMap;

pub struct MinecraftLaunchArgs {
    pub jvm_args: Vec<String>,
    pub main_class: String,
    pub game_args: HashMap<String, String>,
    pub classpath: String,
}