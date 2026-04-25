use crate::domain::launcher::{CheckDirs, FileInfo};
use std::fmt::Debug;
use uuid::Uuid;
use crate::domain::download_url::DownloadUrl;

#[derive(Debug)]
pub struct Profile {
    id: Uuid,
    name: String,
    icon: DownloadUrl,
    hard_check: CheckDirs,
    soft_check: CheckDirs,
}

impl Profile {
    pub fn new(
        id: Uuid,
        name: String,
        icon: DownloadUrl,
        hard_check: CheckDirs,
        soft_check: CheckDirs,
    ) -> Self {
        Self {
            id,
            name,
            icon,
            hard_check,
            soft_check,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::download_url::DownloadUrl;
    use crate::domain::file_path::FilePath;
    use crate::domain::launcher::check::CheckType;
    use std::collections::HashMap;
    use crate::domain::launcher::FileInfo;

    #[test]
    fn test1() {
        let hard_check = FileInfo {
            sha1: vec![],
            _type: CheckType::Required,
            path: FilePath::from_string("Gaga".to_string()),
            url: DownloadUrl::try_from("https://example.com".to_string()).unwrap(),
        };

        let soft_check = FileInfo {
            sha1: vec![],
            _type: CheckType::Required,
            path: FilePath::from_string("Gaga".to_string()),
            url: DownloadUrl::try_from("https://example.com".to_string()).unwrap(),
        };

        let mut map: HashMap<String, FileInfo> = HashMap::new();
        map.insert("my1".to_string(), hard_check);

        let mut map2: HashMap<String, FileInfo> = HashMap::new();
        map2.insert("my2".to_string(), soft_check);

    }
}
