
#[derive(Clone, Debug)]
pub struct Version(semver::Version);

impl From<Version> for String {
    fn from(value: Version) -> Self {
        value.0.to_string()
    }
}

impl TryFrom<String> for Version {
    type Error = VersionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Version(semver::Version::parse(&value).map_err(Self::Error::Parse)?))
    }
}

#[derive(Debug)]
pub enum VersionError {
    Parse(semver::Error),
}
