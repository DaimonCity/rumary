use crate::domain::launcher::value_object::file_path::FilePath;
use crate::domain::launcher::value_object::download_url::DownloadUrl;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CheckDirs {
    pub dirs: HashMap<String, Files>,
}

#[derive(Debug)]
pub struct Files(HashMap<String, FileInfo>);

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub sha1: Vec<u8>,
    pub _type: CheckType,
    pub path: FilePath,
    pub url: DownloadUrl,
}

impl FileInfo {
    pub fn new(sha1: Vec<u8>, _type: CheckType, path: FilePath, url: DownloadUrl) -> Self {
        Self {
            sha1,
            _type,
            path,
            url,
        }
    }
}

impl Files {
    pub fn new(files: HashMap<String, FileInfo>) -> Self {
        Self(files)
    }
}

#[derive(Debug, Clone)]
pub enum CheckType {
    Required,
    Optional,
}
