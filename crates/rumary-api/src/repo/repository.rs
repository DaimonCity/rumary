use async_trait::async_trait;
use rumary_dto::domain::api::{Configuration, Instance, NewConfiguration, NewInstance, NewRole, Role, UpdateConfiguration, UpdateInstance};
use rumary_dto::domain::api::{
    NewTotpUser, NewUser, RefreshSessionUpdate, TotpUser, User, UserSession,
};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use rumary_dto::domain::configuration::ConfigurationId;
use rumary_dto::domain::instance::InstanceId;
use rumary_dto::domain::user::UserId;

#[async_trait]
pub trait UserRepository: Send + Sync {
    type Error;
    async fn create_user(&self, user: NewUser) -> Result<User, Self::Error>;
    async fn find_user(&self, user_id: UserId) -> Result<Option<User>, Self::Error>;
    async fn find_user_by_login(&self, login: &str) -> Result<Option<User>, Self::Error>;
    async fn delete_user(&self, user_id: UserId) -> Result<(), Self::Error>;
    async fn users_list(&self) -> Result<Vec<User>, Self::Error>;
}

#[async_trait]
pub trait InstanceRepository: Send + Sync {
    type Error;
    async fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error>;
    async fn update_instance(
        &self,
        update_instance: UpdateInstance,
    ) -> Result<Instance, Self::Error>;
    async fn get_instance(&self, id: InstanceId, access_level: u16) -> Result<Instance, Self::Error>;
    async fn delete_instance(&self, id: InstanceId) -> Result<(), Self::Error>;
    async fn list_instances(&self, access_level: u16) -> Result<Vec<Instance>, Self::Error>;
}

#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    type Error;
    async fn create_config(&self, new_config: NewConfiguration) -> Result<Configuration, Self::Error>;
    async fn update_config(
        &self,
        id: ConfigurationId,
        update_instance: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error>;
    async fn get_config(&self, id: ConfigurationId, access_level: u16) -> Result<Configuration, Self::Error>;
    async fn delete_config(&self, id: ConfigurationId) -> Result<(), Self::Error>;
    async fn list_configs(&self, instance_id: InstanceId, access_level: u16) -> Result<Vec<Configuration>, Self::Error>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    type Error;
    async fn find_user_by_token_id(
        &self,
        token_uuid: Uuid,
    ) -> Result<Option<UserSession>, Self::Error>;
    async fn save_refresh_session(
        &self,
        user_id: UserId,
        session: RefreshSessionUpdate,
    ) -> Result<(), Self::Error>;
    async fn clear_refresh_session(&self, user_id: UserId) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait TotpRepository: Send + Sync {
    type Error;
    async fn create_totp_user(&self, user: NewTotpUser) -> Result<TotpUser, Self::Error>;
    async fn totp_user_confirmed(&self, user_id: UserId) -> Result<TotpUser, Self::Error>;
    async fn find_totp_user(&self, user_id: UserId) -> Result<Option<TotpUser>, Self::Error>;
    async fn delete_totp_user(&self, user_id: UserId) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    type Error;
    async fn save_instance_dir_path(&self, path: &Path) -> Result<(), Self::Error>;
    async fn get_instances_dir_path(&self) -> Result<PathBuf, Self::Error>;
    async fn delete_instances_dir_path(&self) -> Result<PathBuf, Self::Error>;
}
///
/// Admin -> Worker 10
/// Writer/Builder -> Worker 5
///
#[async_trait]
pub trait RolesRepository: Send + Sync {
    type Error;
    async fn create_role(&self, new_role: NewRole) -> Result<Role, Self::Error>;
    async fn update_role(&self) -> Result<Role, Self::Error>;
    async fn get_role(&self) -> Result<Role, Self::Error>;
    async fn delete_role(&self) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DiscordUserRepository: Send + Sync {}
