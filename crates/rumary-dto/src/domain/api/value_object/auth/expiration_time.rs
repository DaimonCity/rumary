use crate::domain::api::value_object::auth::errors::ExpirationTimeError;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct ExpirationTime(DateTime<Utc>);

impl ExpirationTime {
    pub fn new(time: DateTime<Utc>) -> Result<Self, ExpirationTimeError> {
        if time <= Utc::now() {
            return Err(ExpirationTimeError::InvalidExpiration);
        }
        Ok(Self(time))
    }

    pub fn is_expired(&self) -> bool {
        self.0 <= Utc::now()
    }

    pub fn from_stored(time: DateTime<Utc>) -> Self {
        Self(time)
    }

    pub fn get(&self) -> DateTime<Utc> {
        self.0
    }
}
