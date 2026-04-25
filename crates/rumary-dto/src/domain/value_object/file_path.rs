use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(PathBuf);

impl FilePath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn from_string(path: String) -> Self {
        Self(PathBuf::from(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }
}

impl Deref for FilePath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}