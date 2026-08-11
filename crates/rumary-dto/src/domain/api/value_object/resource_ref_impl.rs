use uuid::Uuid;
use crate::domain::api::{Configuration, Instance, User};
use crate::domain::perms::ResourceRef;
use crate::domain::perms::value_object::resource::{ResourceId, ResourceType, ResourceTypeError};

impl Instance {
    pub fn resource_ref(&self) -> Result<ResourceRef, ResourceTypeError> {
        Ok(ResourceRef::new(
            ResourceType::try_from("instance")?,
            ResourceId::from(Uuid::from(self.id)),
        ))
    }
}

impl Configuration {
    pub fn resource_ref(&self) -> Result<ResourceRef, ResourceTypeError> {
        Ok(ResourceRef::new(
            ResourceType::try_from("configuration")?,
            ResourceId::from(Uuid::from(self.id)),
        ))
    }
}

impl User {
    pub fn resource_ref(&self) -> Result<ResourceRef, ResourceTypeError> {
        Ok(ResourceRef::new(
            ResourceType::try_from("user")?,
            ResourceId::from(Uuid::from(self.id)),
        ))
    }
}