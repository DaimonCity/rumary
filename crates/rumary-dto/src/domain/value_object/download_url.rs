use std::ops::Deref;
use url::{ParseError, Url};

#[derive(Debug, Clone)]
pub struct DownloadUrl(Url);

impl DownloadUrl {
    pub fn new(url: Url) -> Self {
        Self(url)
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn from_string(string: String) -> Result<DownloadUrl, ParseError> {
        Self::try_from(string)
    }
}


impl TryFrom<String> for DownloadUrl {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let url = Url::parse(&value)?;
        Ok(Self(url))
    }
}
impl TryFrom<&str> for DownloadUrl {
    type Error = url::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(value)?;
        Ok(Self(url))
    }
}

impl Deref for DownloadUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}