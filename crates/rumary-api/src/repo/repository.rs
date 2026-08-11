use async_trait::async_trait;
use rumary_dto::domain::api::{
    Configuration, Instance, NewConfiguration, NewInstance, UpdateConfiguration, UpdateInstance,
};
use rumary_dto::domain::api::{
    BanId, NewTotpUser, NewUser, NewUserBan, RefreshSessionUpdate, TotpUser, User, UserBan,
    UserSession,
};
use rumary_dto::domain::api::value_object::auth::tokens::TokenId;
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::domain::api::value_object::instance::InstanceId;
use rumary_dto::domain::api::value_object::user::{Login, UserId};
use std::path::{Path, PathBuf};

#[async_trait]
pub trait UserRepository: Send + Sync {
    type Error;
    async fn create_user(&self, user: NewUser) -> Result<User, Self::Error>;
    async fn find_user(&self, user_id: UserId) -> Result<Option<User>, Self::Error>;
    async fn find_user_by_login(&self, login: Login) -> Result<Option<User>, Self::Error>;
    async fn delete_user(&self, user_id: UserId) -> Result<(), Self::Error>;
    async fn list_users(&self) -> Result<Vec<User>, Self::Error>;
}

#[async_trait]
pub trait InstanceRepository: Send + Sync {
    type Error;
    async fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error>;
    async fn update_instance(
        &self,
        instance_id: InstanceId,
        update_instance: UpdateInstance,
    ) -> Result<Instance, Self::Error>;
    async fn get_instance(&self, id: InstanceId) -> Result<Instance, Self::Error>;
    async fn delete_instance(&self, id: InstanceId) -> Result<Instance, Self::Error>;
    async fn list_instances(&self) -> Result<Vec<Instance>, Self::Error>;
}

#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    type Error;
    async fn create_config(
        &self,
        new_config: NewConfiguration,
    ) -> Result<Configuration, Self::Error>;
    async fn update_config(
        &self,
        id: ConfigurationId,
        update_instance: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error>;
    async fn get_config(&self, id: ConfigurationId) -> Result<Configuration, Self::Error>;
    async fn delete_config(&self, id: ConfigurationId) -> Result<Configuration, Self::Error>;
    async fn list_for_instance(
        &self,
        instance_id: InstanceId,
    ) -> Result<Vec<Configuration>, Self::Error>;
    async fn list_all_configs(&self) -> Result<Vec<Configuration>, Self::Error>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    type Error;
    async fn find_user_by_token_id(
        &self,
        token_id: TokenId,
    ) -> Result<Option<UserSession>, Self::Error>;
    async fn save_refresh_session(
        &self,
        user_id: UserId,
        session: RefreshSessionUpdate,
    ) -> Result<(), Self::Error>;
    async fn clear_refresh_session(&self, user_id: UserId) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait ModerationRepository: Send + Sync {
    type Error;

    async fn create_user_ban_and_revoke_sessions(
        &self,
        ban: NewUserBan,
    ) -> Result<UserBan, Self::Error>;

    async fn find_active_api_ban(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserBan>, Self::Error>;

    async fn list_user_bans(&self, user_id: UserId) -> Result<Vec<UserBan>, Self::Error>;

    async fn revoke_user_ban(
        &self,
        ban_id: BanId,
        user_id: UserId,
        revoked_by: UserId,
        reason: String,
    ) -> Result<Option<UserBan>, Self::Error>;
}

#[async_trait]
pub trait TotpRepository: Send + Sync {
    type Error;
    async fn create_totp_user(&self, user: NewTotpUser) -> Result<TotpUser, Self::Error>;
    async fn totp_user_enable(&self, user_id: UserId) -> Result<TotpUser, Self::Error>;
    async fn find_totp_user(&self, user_id: UserId) -> Result<Option<TotpUser>, Self::Error>;
    async fn totp_user_disable(&self, user_id: UserId) -> Result<Option<TotpUser>, Self::Error>;
    async fn delete_totp_user(&self, user_id: UserId) -> Result<(), Self::Error>;
    async fn save_used_step_if_newer(&self, user_id: UserId, step: i64) -> Result<bool, Self::Error>;
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
pub trait DiscordUserRepository: Send + Sync {}

// #[async_trait]
// pub trait StorageRepository: Send + Sync {
//     async fn get_file_stream()
// }
