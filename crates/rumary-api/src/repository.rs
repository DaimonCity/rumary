use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{Client, InstallationRequest, LauncherBuild, Profile, Session, User},
};

#[async_trait]
pub trait AppRepository: Send + Sync {
    async fn insert_user(&self, user: &User) -> AppResult<()>;
    async fn find_user_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_user_by_id(&self, user_id: Uuid) -> AppResult<Option<User>>;
    async fn update_user_ban(&self, user_id: Uuid, banned: bool) -> AppResult<User>;
    async fn list_users(&self) -> AppResult<Vec<User>>;
    async fn insert_session(&self, session: &Session) -> AppResult<()>;

    async fn insert_client(&self, client: &Client) -> AppResult<()>;
    async fn list_clients(&self) -> AppResult<Vec<Client>>;
    async fn find_client_by_id(&self, client_id: Uuid) -> AppResult<Option<Client>>;

    async fn insert_profile(&self, profile: &Profile) -> AppResult<()>;
    async fn find_profile_by_id(&self, profile_id: Uuid) -> AppResult<Option<Profile>>;

    async fn insert_launcher_build(&self, build: &LauncherBuild) -> AppResult<()>;
    async fn latest_launcher_build(&self, channel: &str) -> AppResult<Option<LauncherBuild>>;

    async fn insert_installation_request(&self, request: &InstallationRequest) -> AppResult<()>;
}
