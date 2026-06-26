use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::external::mojang::dto::response::download::DownloadInfo;
use crate::external::mojang::dto::response::rule::Rule;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct LibraryDownload {
//     pub artifact: Option<DownloadInfo>,
//     pub classifiers: Option<HashMap<String, DownloadInfo>>,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LibraryDownloads {
    Artifact(Option<DownloadInfo>),
    Classifiers(Option<HashMap<String, DownloadInfo>>),
}