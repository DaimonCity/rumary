pub struct IconUrl(String);

impl IconUrl {
    const EMPTY: IconUrl = IconUrl(String::new());
}
impl TryFrom<String> for IconUrl {
    type Error = IconUrlError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self::EMPTY);
        }

        if !value.contains("http") {
            return Err(Self::Error::InvalidUrl);
        }

        Ok(IconUrl(value))
    }
}

impl From<IconUrl> for String {
    fn from(url: IconUrl) -> Self {
        url.0
    }
}

#[derive(Debug)]
pub enum IconUrlError {
    InvalidUrl,
}
