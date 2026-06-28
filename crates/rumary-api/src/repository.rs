use crate::error::AppResult;
use async_trait::async_trait;
use rumary_dto::domain::api::{
    NewTotpUser, NewUser, RefreshSessionUpdate, TotpUser, User, UserSession,
};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: NewUser) -> AppResult<User>;
    async fn find_user(&self, uuid: Uuid) -> AppResult<Option<User>>;
    async fn find_user_by_login(&self, login: &str) -> AppResult<Option<User>>;
    async fn delete_user(&self, uuid: Uuid) -> AppResult<()>;
    async fn users_list(&self) -> AppResult<Vec<User>>;
}
use rumary_dto::domain::api::{
    Configuration, Instance, NewConfiguration, NewInstance, UpdateConfiguration, UpdateInstance,
};

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
pub trait SessionRepository: Send + Sync {
    async fn find_user_by_token_id(&self, token_uuid: Uuid) -> AppResult<Option<UserSession>>;
    async fn save_refresh_session(
        &self,
        user_uuid: Uuid,
        session: RefreshSessionUpdate,
    ) -> AppResult<()>;
    async fn clear_refresh_session(&self, user_uuid: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait ConfigurationRepo: Send + Sync {
    type Error;
    fn create_config(&self, new_config: NewConfiguration) -> Result<Configuration, Self::Error>;
    fn update_config(
        &self,
        update_instance: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error>;
    fn find_config(&self, uuid: Uuid) -> Result<Configuration, Self::Error>;
    fn delete_config(&self, uuid: Uuid) -> Result<(), Self::Error>;
    fn get_list_configs(&self, access_level: u16) -> Result<Vec<Configuration>, Self::Error>;
}

#[async_trait]
pub trait TotpRepository: Send + Sync {
    async fn create_totp_user(&self, user: NewTotpUser) -> AppResult<TotpUser>;
    async fn totp_user_confirmed(&self, uuid: Uuid) -> AppResult<TotpUser>;
    async fn find_totp_user(&self, uuid: Uuid) -> AppResult<Option<TotpUser>>;
    async fn delete_totp_user(&self, uuid: Uuid) -> AppResult<()>;
}
