use std::path::{Path, PathBuf};
use async_trait::async_trait;
use rumary_dto::domain::api::{
    NewTotpUser, NewUser, RefreshSessionUpdate, TotpUser, User, UserSession,
};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    type Error;
    async fn create_user(&self, user: NewUser) -> Result<User, Self::Error>;
    async fn find_user(&self, uuid: Uuid) -> Result<Option<User>, Self::Error>;
    async fn find_user_by_login(&self, login: &str) -> Result<Option<User>, Self::Error>;
    async fn delete_user(&self, uuid: Uuid) -> Result<(), Self::Error>;
    async fn users_list(&self) -> Result<Vec<User>, Self::Error>;
}
use rumary_dto::domain::api::{
    Configuration, Instance, NewConfiguration, NewInstance, UpdateConfiguration, UpdateInstance,
};

#[async_trait]
pub trait InstanceRepository: Send + Sync {
    type Error;
    async fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error>;
    async fn update_instance(&self, update_instance: UpdateInstance) -> Result<Instance, Self::Error>;
    async fn find_instance(&self, uuid: Uuid) -> Result<Instance, Self::Error>;
    async fn delete_instance(&self, uuid: Uuid) -> Result<(), Self::Error>;
    async fn list_instances(&self, access_level: u16) -> Result<Vec<Instance>, Self::Error>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    type Error;
    async fn find_user_by_token_id(&self, token_uuid: Uuid) -> Result<Option<UserSession>, Self::Error>;
    async fn save_refresh_session(
        &self,
        user_uuid: Uuid,
        session: RefreshSessionUpdate,
    ) -> Result<(), Self::Error>;
    async fn clear_refresh_session(&self, user_uuid: Uuid) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    type Error;
    async fn create_config(&self, new_config: NewConfiguration) -> Result<Configuration, Self::Error>;
    async fn update_config(
        &self,
        update_instance: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error>;
    async fn find_config(&self, uuid: Uuid) -> Result<Configuration, Self::Error>;
    async fn delete_config(&self, uuid: Uuid) -> Result<(), Self::Error>;
    async fn get_list_configs(&self, access_level: u16) -> Result<Vec<Configuration>, Self::Error>;
}

#[async_trait]
pub trait TotpRepository: Send + Sync {
    type Error;
    async fn create_totp_user(&self, user: NewTotpUser) -> Result<TotpUser, Self::Error>;
    async fn totp_user_confirmed(&self, uuid: Uuid) -> Result<TotpUser,  Self::Error>;
    async fn find_totp_user(&self, uuid: Uuid) -> Result<Option<TotpUser>, Self::Error>;
    async fn delete_totp_user(&self, uuid: Uuid) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    type Error;
    async fn save_instance_dir_path(&self, path: &Path) -> Result<(), Self::Error>;
    async fn save_configuration_dir_path(&self, path: &Path) -> Result<(), Self::Error>;
    async fn get_instance_dir_path(&self) -> Result<PathBuf, Self::Error>;
    async fn get_configuration_dir_path(&self) -> Result<PathBuf, Self::Error>;
    async fn delete_totp_user(&self, uuid: Uuid) -> Result<(), Self::Error>;
}
