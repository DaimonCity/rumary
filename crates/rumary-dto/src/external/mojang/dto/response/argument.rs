use serde::{Deserialize, Serialize};
use crate::mojang::dto::response::Rule;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Complex {
        rules: Option<Vec<Rule>>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}