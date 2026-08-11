use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListGroupsQuery {
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: u32,
}