use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};
use crate::repo::repository::{
    ConfigurationRepository, InstanceRepository, SessionRepository, SettingsRepository,
    TotpRepository, UserRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumary_dto::domain::api::{
    Configuration, Instance, NewConfiguration, NewInstance, NewTotpUser, NewUser,
    RefreshSessionUpdate, TotpUser, UpdateConfiguration, UpdateInstance, User, UserSession,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Pool, Postgres, Row};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepo {
    pool: PgPool,
}

impl PostgresRepo {
    pub async fn connect(config: DatabaseConfig) -> AppResult<Self> {
        let options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.user)
            .password(&config.password)
            .database(&config.db_name);

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(8))
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> Pool<Postgres> {
        self.pool.clone()
    }
}
#[async_trait]
impl UserRepository for PostgresRepo {
    type Error = AppError;

    async fn create_user(&self, user: NewUser) -> AppResult<User> {
        todo!()
    }

    async fn find_user(&self, uuid: Uuid) -> AppResult<Option<User>> {
        todo!()
    }

    async fn find_user_by_login(&self, login: &str) -> AppResult<Option<User>> {
        todo!()
    }

    async fn delete_user(&self, uuid: Uuid) -> AppResult<()> {
        todo!()
    }

    async fn users_list(&self) -> AppResult<Vec<User>> {
        todo!()
    }
}
#[async_trait]
impl TotpRepository for PostgresRepo {
    type Error = AppError;

    async fn create_totp_user(&self, user: NewTotpUser) -> AppResult<TotpUser> {
        todo!()
    }

    async fn totp_user_confirmed(&self, uuid: Uuid) -> AppResult<TotpUser> {
        todo!()
    }

    async fn find_totp_user(&self, uuid: Uuid) -> AppResult<Option<TotpUser>> {
        todo!()
    }

    async fn delete_totp_user(&self, uuid: Uuid) -> AppResult<()> {
        todo!()
    }
}
#[async_trait]
impl SessionRepository for PostgresRepo {
    type Error = AppError;

    async fn find_user_by_token_id(&self, token_uuid: Uuid) -> AppResult<Option<UserSession>> {
        todo!()
    }

    async fn save_refresh_session(
        &self,
        user_uuid: Uuid,
        session: RefreshSessionUpdate,
    ) -> AppResult<()> {
        todo!()
    }

    async fn clear_refresh_session(&self, user_uuid: Uuid) -> AppResult<()> {
        todo!()
    }
}

#[async_trait]
impl SettingsRepository for PostgresRepo {
    type Error = AppError;

    async fn save_instance_dir_path(&self, path: &Path) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_instances_dir_path(&self) -> Result<PathBuf, Self::Error> {
        todo!()
    }

    async fn delete_instances_dir_path(&self) -> Result<PathBuf, Self::Error> {
        todo!()
    }
}
#[async_trait]
impl InstanceRepository for PostgresRepo {
    type Error = AppError;

    async fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error> {
        todo!()
    }

    async fn update_instance(
        &self,
        update_instance: UpdateInstance,
    ) -> Result<Instance, Self::Error> {
        todo!()
    }

    async fn find_instance(&self, uuid: Uuid) -> Result<Instance, Self::Error> {
        todo!()
    }

    async fn delete_instance(&self, uuid: Uuid) -> Result<(), Self::Error> {
        todo!()
    }

    async fn list_instances(&self, access_level: u16) -> Result<Vec<Instance>, Self::Error> {
        todo!()
    }
}
#[async_trait]
impl ConfigurationRepository for PostgresRepo {
    type Error = AppError;

    async fn create_config(
        &self,
        new_config: NewConfiguration,
    ) -> Result<Configuration, Self::Error> {
        todo!()
    }

    async fn update_config(
        &self,
        update_instance: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error> {
        todo!()
    }

    async fn find_config(&self, uuid: Uuid) -> Result<Configuration, Self::Error> {
        todo!()
    }

    async fn delete_config(&self, uuid: Uuid) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_list_configs(&self, access_level: u16) -> Result<Vec<Configuration>, Self::Error> {
        todo!()
    }
}

fn _json<T: Serialize>(value: &T) -> AppResult<sqlx::types::Json<serde_json::Value>> {
    serde_json::to_value(value)
        .map(sqlx::types::Json)
        .map_err(|err| AppError::Internal(err.to_string()))
}

fn _from_json<T: DeserializeOwned>(value: sqlx::types::Json<serde_json::Value>) -> AppResult<T> {
    serde_json::from_value(value.0).map_err(|err| AppError::Database(err.to_string()))
}

fn _map_conflict(entity: &'static str) -> impl Fn(sqlx::Error) -> AppError {
    move |err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            AppError::Conflict(format!("{entity} already exists"))
        }
        _ => AppError::Database(err.to_string()),
    }
}

#[allow(dead_code)]
fn _row_to_timestamp(
    row: &sqlx::postgres::PgRow,
    field: &str,
) -> Result<DateTime<Utc>, sqlx::Error> {
    row.try_get(field)
}
