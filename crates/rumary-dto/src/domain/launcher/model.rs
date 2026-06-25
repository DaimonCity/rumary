use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftLaunchArgs {
    pub jvm_args: Vec<String>,
    pub main_class: String,
    pub game_args: HashMap<String, String>,
    pub classpath: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChosenVersion {
    pub id: Uuid,
    pub name: String,
    pub url: String,
}