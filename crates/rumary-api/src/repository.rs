use async_trait::async_trait;
use uuid::Uuid;
use rumary_dto::domain::api::{Configuration, Instance, NewConfiguration, NewInstance, UpdateConfiguration, UpdateInstance};

#[async_trait]
pub trait InstanceRepo: Send + Sync {
    type Error;
    fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error>;
    fn update_instance(&self, update_instance: UpdateInstance) -> Result<Instance, Self::Error>;
    fn find_instance(&self, uuid: Uuid) -> Result<Instance, Self::Error>;
    fn delete_instance(&self, uuid: Uuid) -> Result<(), Self::Error>;
    fn get_list_configs(&self, access_level: u16) -> Result<Vec<Instance>, Self::Error>;
}

#[async_trait]
pub trait ConfigurationRepo: Send + Sync {
    type Error;
    fn create_config(&self, new_config: NewConfiguration) -> Result<Configuration, Self::Error>;
    fn update_config(&self, update_instance: UpdateConfiguration) -> Result<Configuration, Self::Error>;
    fn find_config(&self, uuid: Uuid) -> Result<Configuration, Self::Error>;
    fn delete_config(&self, uuid: Uuid) -> Result<(), Self::Error>;
    fn get_list_configs(&self, access_level: u16) -> Result<Vec<Configuration>, Self::Error>;
}
