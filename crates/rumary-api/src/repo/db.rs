use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};
use crate::repo::repository::{
    ConfigurationRepository, DiscordUserRepository, InstanceRepository, SessionRepository,
    SettingsRepository, TotpRepository, UserRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumary_dto::domain::api::value_object::auth::tokens::TokenHash;
use rumary_dto::domain::api::value_object::auth::tokens::TokenId;
use rumary_dto::domain::api::value_object::configuration::ConfigurationId;
use rumary_dto::domain::api::value_object::instance::InstanceId;
use rumary_dto::domain::api::value_object::name::{Description, DirectoryName, DisplayName};
use rumary_dto::domain::api::value_object::url::IconUrl;
use rumary_dto::domain::api::value_object::user::PasswordHash;
use rumary_dto::domain::api::value_object::user::{Login, UserId};
use rumary_dto::domain::api::value_object::version::Version;
use rumary_dto::domain::api::{
    Configuration, Instance, Loader, NewConfiguration, NewInstance, NewTotpUser, NewUser,
    RefreshSessionUpdate, TotpUser, UpdateConfiguration, UpdateInstance, User, UserSession,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Pool, Postgres};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepo {
    pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    user_id: Uuid,
    login: String,
    nickname: String,
    password_hash: String,
    token_version: i32,
    is_public: bool,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> AppResult<Self> {
        Ok(Self {
            id: UserId::from(row.user_id),
            login: row.login.try_into()?,
            nickname: row.nickname.try_into()?,
            password_hash: PasswordHash::from_stored(row.password_hash),
            token_version: row.token_version,
            is_public: row.is_public,
        })
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    user_id: Uuid,
    token_id: Uuid,
    refresh_token_hash: String,
    expires_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct TotpRow {
    user_id: Uuid,
    encrypted_secret: String,
    step: i64,
    nonce: String,
    confirmed: bool,
}

impl From<TotpRow> for TotpUser {
    fn from(row: TotpRow) -> Self {
        Self {
            id: UserId::from(row.user_id),
            totp: row.encrypted_secret,
            nonce: row.nonce,
            step: row.step,
            confirmed: row.confirmed,
        }
    }
}

#[derive(sqlx::FromRow)]
struct InstanceRow {
    id: Uuid,
    icon: String,
    dir_name: String,
    display_name: String,
    version: String,
    description: String,
    loader: String,
    loader_version: Option<String>,
    is_public: bool,
}

impl TryFrom<InstanceRow> for Instance {
    type Error = AppError;

    fn try_from(row: InstanceRow) -> AppResult<Self> {
        let loader_version = row.loader_version.map(Version::try_from).transpose()?;

        Ok(Self {
            id: InstanceId::from(row.id),
            icon: IconUrl::try_from(row.icon)?,
            dir_name: DirectoryName::try_from(row.dir_name)?,
            display_name: DisplayName::try_from(row.display_name)?,
            version: Version::try_from(row.version)?,
            description: Description::try_from(row.description)?,
            loader: Loader::from_string(row.loader, loader_version)?,
            is_public: row.is_public,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ConfigurationRow {
    id: Uuid,
    icon: Option<String>,
    dir_name: String,
    display_name: String,
    instance_id: Uuid,
    is_public: bool,
}

impl TryFrom<ConfigurationRow> for Configuration {
    type Error = AppError;

    fn try_from(row: ConfigurationRow) -> AppResult<Self> {
        let icon = row.icon.map(TryFrom::try_from).transpose()?;
        Ok(Self {
            id: ConfigurationId::from(row.id),
            icon,
            dir_name: DirectoryName::try_from(row.dir_name)?,
            display_name: DisplayName::try_from(row.display_name)?,
            instance_id: InstanceId::from(row.instance_id),
            is_public: row.is_public,
        })
    }
}

#[allow(unused)]
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

    /// Пул без клонирования — для запросов внутри крейта (см. `repo::perms`).
    pub(crate) fn pool_ref(&self) -> &PgPool {
        &self.pool
    }
}
#[allow(unused)]
#[async_trait]
impl UserRepository for PostgresRepo {
    type Error = AppError;

    async fn create_user(&self, user: NewUser) -> AppResult<User> {
        let mut tx = self.pool.begin().await?;
        let row: UserRow = sqlx::query_as(
            r#"
            INSERT INTO users (login, nickname, password_hash, is_public)
            VALUES ($1, $2, $3, false)
            RETURNING user_id, login, nickname, password_hash, token_version, is_public
            "#,
        )
        .bind(String::from(user.login))
        .bind(String::from(user.nickname))
        .bind(String::from(user.password_hash))
        .fetch_one(&mut *tx)
        .await
        .map_err(_map_conflict("user"))?;

        sqlx::query(
            "INSERT INTO user_groups (user_id, group_name) VALUES ($1, 'user') ON CONFLICT DO NOTHING",
        )
        .bind(row.user_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        row.try_into()
    }

    async fn find_user(&self, user_id: UserId) -> AppResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT user_id, login, nickname, password_hash, token_version, is_public FROM users WHERE user_id = $1",
        )
        .bind(Uuid::from(user_id))
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn find_user_by_login(&self, login: Login) -> AppResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT user_id, login, nickname, password_hash, token_version, is_public FROM users WHERE login = $1",
        )
        .bind(login.as_str())
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn delete_user(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE user_id = $1")
            .bind(Uuid::from(user_id))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        let rows: Vec<UserRow> = sqlx::query_as(
            "SELECT user_id, login, nickname, password_hash, token_version, is_public FROM users ORDER BY login",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
#[allow(unused)]
#[async_trait]
impl TotpRepository for PostgresRepo {
    type Error = AppError;

    async fn create_totp_user(&self, user: NewTotpUser) -> AppResult<TotpUser> {
        let row: TotpRow = sqlx::query_as(
            r#"
            INSERT INTO user_totp (user_id, encrypted_secret, nonce, confirmed)
            VALUES ($1, $2, $3, false)
            ON CONFLICT (user_id) DO UPDATE
            SET encrypted_secret = EXCLUDED.encrypted_secret,
                nonce = EXCLUDED.nonce,
                confirmed = false,
                updated_at = now()
            RETURNING user_id, encrypted_secret, nonce, confirmed, step
            "#,
        )
        .bind(user.user_id)
        .bind(user.encrypted_secret)
        .bind(user.nonce)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn totp_user_enable(&self, user_id: UserId) -> AppResult<TotpUser> {
        let row: Option<TotpRow> = sqlx::query_as(
            "UPDATE user_totp SET confirmed = true, updated_at = now() WHERE user_id = $1 RETURNING user_id, encrypted_secret, nonce, step, confirmed",
        )
        .bind(Uuid::from(user_id))
        .fetch_optional(&self.pool)
        .await?;
        row.map(Into::into)
            .ok_or_else(|| AppError::NotFound(format!("totp user {user_id}")))
    }

    async fn find_totp_user(&self, user_id: UserId) -> AppResult<Option<TotpUser>> {
        let row: Option<TotpRow> = sqlx::query_as(
            "SELECT user_id, encrypted_secret, nonce, confirmed, step FROM user_totp WHERE user_id = $1",
        )
        .bind(Uuid::from(user_id))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn totp_user_disable(&self, user_id: UserId) -> Result<Option<TotpUser>, Self::Error> {
        let row: Option<TotpRow> = sqlx::query_as(
            "UPDATE user_totp SET confirmed = false, updated_at = now() WHERE user_id = $1 RETURNING user_id, encrypted_secret, nonce, step, confirmed",
        )
        .bind(Uuid::from(user_id))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete_totp_user(&self, user_id: UserId) -> AppResult<()> {
        sqlx::query("DELETE FROM user_totp WHERE user_id = $1")
            .bind(Uuid::from(user_id))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_used_step_if_newer(
        &self,
        user_id: UserId,
        step: i64,
    ) -> Result<bool, Self::Error> {
        let result = sqlx::query("UPDATE user_totp SET step = $1 WHERE user_id = $2 AND step < $1")
            .bind(step)
            .bind(Uuid::from(user_id))
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
#[allow(unused)]
#[async_trait]
impl SessionRepository for PostgresRepo {
    type Error = AppError;

    async fn find_user_by_token_id(&self, token_id: TokenId) -> AppResult<Option<UserSession>> {
        let row: Option<SessionRow> = sqlx::query_as(
            "SELECT user_id AS user_id, token_id, refresh_token_hash, expires_at FROM user_sessions WHERE token_id = $1",
        )
        .bind(Uuid::from(token_id))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| UserSession {
            id: UserId::from(row.user_id),
            token_id: TokenId::from(row.token_id),
            refresh_token_hash: TokenHash::from_stored(row.refresh_token_hash),
            expires_at: row.expires_at,
        }))
    }

    async fn save_refresh_session(
        &self,
        user_id: UserId,
        session: RefreshSessionUpdate,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO user_sessions (
                user_id, token_id, refresh_token_hash, expires_at, created_at, updated_at
            )
            SELECT $1, $2, $3, $4, now(), now()
            FROM users
            WHERE user_id = $1
            ON CONFLICT (user_id) DO UPDATE
            SET token_id = EXCLUDED.token_id,
                refresh_token_hash = EXCLUDED.refresh_token_hash,
                expires_at = EXCLUDED.expires_at,
                updated_at = now()
            "#,
        )
        .bind(Uuid::from(user_id))
        .bind(Uuid::from(session.token_id))
        .bind(String::from(session.refresh_token_hash))
        .bind(session.expires_at.get())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("user {user_id}")));
        }
        Ok(())
    }

    async fn clear_refresh_session(&self, user_id: UserId) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let user_id = Uuid::from(user_id);

        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE users SET token_version = token_version + 1, updated_at = now() WHERE user_id = $1",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[allow(unused)]
#[async_trait]
impl SettingsRepository for PostgresRepo {
    type Error = AppError;

    async fn save_instance_dir_path(&self, path: &Path) -> Result<(), Self::Error> {
        let path = path
            .to_str()
            .filter(|path| !path.is_empty())
            .ok_or_else(|| {
                AppError::Validation("instance directory path must be non-empty UTF-8".to_owned())
            })?;

        sqlx::query(
            r#"
            INSERT INTO settings (singleton, instances_dir_path, updated_at)
            VALUES (true, $1, now())
            ON CONFLICT (singleton) DO UPDATE
            SET instances_dir_path = EXCLUDED.instances_dir_path,
                updated_at = now()
            "#,
        )
        .bind(path)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_instances_dir_path(&self) -> Result<PathBuf, Self::Error> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT instances_dir_path FROM settings WHERE singleton = true")
                .fetch_optional(&self.pool)
                .await?;

        row.map(|(path,)| PathBuf::from(path))
            .ok_or_else(|| AppError::NotFound("instance directory path".to_owned()))
    }

    async fn delete_instances_dir_path(&self) -> Result<PathBuf, Self::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "DELETE FROM settings WHERE singleton = true RETURNING instances_dir_path",
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|(path,)| PathBuf::from(path))
            .ok_or_else(|| AppError::NotFound("instance directory path".to_owned()))
    }
}
#[allow(unused)]
#[async_trait]
impl InstanceRepository for PostgresRepo {
    type Error = AppError;

    async fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Self::Error> {
        let NewInstance {
            icon,
            dir_name,
            display_name,
            version,
            description,
            loader,
            is_public,
        } = new_instance;
        let (loader, loader_version) = _loader_parts(loader);

        let row: InstanceRow = sqlx::query_as(
            r#"
            INSERT INTO instances (
                icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            "#,
        )
        .bind(String::from(icon))
        .bind(String::from(dir_name))
        .bind(String::from(display_name))
        .bind(String::from(version))
        .bind(String::from(description))
        .bind(loader)
        .bind(loader_version)
        .bind(is_public)
        .fetch_one(&self.pool)
        .await
        .map_err(_map_conflict("instance"))?;

        row.try_into()
    }

    async fn update_instance(
        &self,
        instance_id: InstanceId,
        update_instance: UpdateInstance,
    ) -> Result<Instance, Self::Error> {
        let UpdateInstance {
            icon,
            dir_name,
            display_name,
            version,
            description,
            loader,
        } = update_instance;
        let (loader, loader_version) = loader
            .map(_loader_parts)
            .map_or((None, None), |(loader, version)| (Some(loader), version));

        let row: Option<InstanceRow> = sqlx::query_as(
            r#"
            UPDATE instances
            SET icon = COALESCE($2, icon),
                dir_name = COALESCE($3, dir_name),
                display_name = COALESCE($4, display_name),
                version = COALESCE($5, version),
                description = COALESCE($6, description),
                loader = COALESCE($7, loader),
                loader_version = CASE WHEN $7 IS NULL THEN loader_version ELSE $8 END,
                updated_at = now()
            WHERE id = $1
            RETURNING
                id, icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            "#,
        )
        .bind(Uuid::from(instance_id))
        .bind(icon.map(String::from))
        .bind(dir_name.map(String::from))
        .bind(display_name.map(String::from))
        .bind(version.map(String::from))
        .bind(description.map(String::from))
        .bind(loader)
        .bind(loader_version)
        .fetch_optional(&self.pool)
        .await
        .map_err(_map_conflict("instance"))?;

        row.ok_or_else(|| AppError::NotFound(format!("instance {instance_id}")))?
            .try_into()
    }

    async fn get_instance(&self, id: InstanceId) -> Result<Instance, Self::Error> {
        let row: Option<InstanceRow> = sqlx::query_as(
            r#"
            SELECT
                id, icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            FROM instances
            WHERE id = $1
            "#,
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound(format!("instance {id}")))?
            .try_into()
    }

    async fn delete_instance(&self, id: InstanceId) -> Result<Instance, Self::Error> {
        let row: Option<InstanceRow> = sqlx::query_as(
            r#"
            DELETE FROM instances
            WHERE id = $1
            RETURNING
                id, icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            "#,
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound(format!("instance {id}")))?
            .try_into()
    }

    async fn list_instances(&self) -> Result<Vec<Instance>, Self::Error> {
        let rows: Vec<InstanceRow> = sqlx::query_as(
            r#"
            SELECT
                id, icon, dir_name, display_name, version, description,
                loader, loader_version, is_public
            FROM instances
            ORDER BY dir_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}
#[allow(unused)]
#[async_trait]
impl ConfigurationRepository for PostgresRepo {
    type Error = AppError;

    async fn create_config(
        &self,
        new_config: NewConfiguration,
    ) -> Result<Configuration, Self::Error> {
        let NewConfiguration {
            icon,
            dir_name,
            display_name,
            instance_id,
            is_public,
        } = new_config;

        let row: ConfigurationRow = sqlx::query_as(
            r#"
            INSERT INTO configurations (
                icon, dir_name, display_name, instance_id, is_public
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, icon, dir_name, display_name, instance_id, is_public
            "#,
        )
        .bind(String::from(icon))
        .bind(String::from(dir_name))
        .bind(String::from(display_name))
        .bind(Uuid::from(instance_id))
        .bind(is_public)
        .fetch_one(&self.pool)
        .await
        .map_err(_map_conflict("configuration"))?;

        row.try_into()
    }

    async fn update_config(
        &self,
        id: ConfigurationId,
        update_config: UpdateConfiguration,
    ) -> Result<Configuration, Self::Error> {
        let UpdateConfiguration {
            icon,
            dir_name,
            display_name,
            instance_id,
        } = update_config;

        let row: Option<ConfigurationRow> = sqlx::query_as(
            r#"
            UPDATE configurations
            SET icon = COALESCE($2, icon),
                dir_name = COALESCE($3, dir_name),
                display_name = COALESCE($4, display_name),
                instance_id = COALESCE($5, instance_id),
                updated_at = now()
            WHERE id = $1
            RETURNING id, icon, dir_name, display_name, instance_id, is_public
            "#,
        )
        .bind(Uuid::from(id))
        .bind(icon.map(String::from))
        .bind(dir_name.map(String::from))
        .bind(display_name.map(String::from))
        .bind(instance_id.map(Uuid::from))
        .fetch_optional(&self.pool)
        .await
        .map_err(_map_conflict("configuration"))?;

        row.ok_or_else(|| AppError::NotFound(format!("configuration {id}")))?
            .try_into()
    }

    async fn get_config(&self, id: ConfigurationId) -> Result<Configuration, Self::Error> {
        let row: Option<ConfigurationRow> = sqlx::query_as(
            r#"
            SELECT id, icon, dir_name, display_name, instance_id, is_public
            FROM configurations
            WHERE id = $1
            "#,
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound(format!("configuration {id}")))?
            .try_into()
    }

    async fn delete_config(&self, id: ConfigurationId) -> Result<Configuration, Self::Error> {
        let row: Option<ConfigurationRow> = sqlx::query_as(
            r#"
            DELETE FROM configurations
            WHERE id = $1
            RETURNING id, icon, dir_name, display_name, instance_id, is_public
            "#,
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| AppError::NotFound(format!("configuration {id}")))?
            .try_into()
    }

    async fn list_for_instance(
        &self,
        instance_id: InstanceId,
    ) -> Result<Vec<Configuration>, Self::Error> {
        let rows: Vec<ConfigurationRow> = sqlx::query_as(
            r#"
            SELECT id, icon, dir_name, display_name, instance_id, is_public
            FROM configurations
            WHERE instance_id = $1
            ORDER BY dir_name, id
            "#,
        )
        .bind(Uuid::from(instance_id))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn list_all_configs(&self) -> Result<Vec<Configuration>, Self::Error> {
        let rows: Vec<ConfigurationRow> = sqlx::query_as(
            r#"
            SELECT id, icon, dir_name, display_name, instance_id, is_public
            FROM configurations
            ORDER BY instance_id, dir_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}

impl DiscordUserRepository for PostgresRepo {}

fn _loader_parts(loader: Loader) -> (String, Option<String>) {
    let version = loader.get_version().map(String::from);
    (String::from(loader), version)
}

fn _map_conflict(entity: &'static str) -> impl Fn(sqlx::Error) -> AppError {
    move |err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            AppError::Conflict(format!("{entity} already exists"))
        }
        _ => AppError::Database(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires TEST_DATABASE_URL pointing to an empty PostgreSQL database"]
    async fn postgres_repository_crud() {
        let database_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("connect test database");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("apply migrations");
        let repo = PostgresRepo { pool };
        let suffix = Uuid::new_v4();

        repo.save_instance_dir_path(Path::new("/tmp/rumary-instances"))
            .await
            .expect("save settings");
        assert_eq!(
            repo.get_instances_dir_path().await.expect("load settings"),
            PathBuf::from("/tmp/rumary-instances")
        );
        assert_eq!(
            repo.delete_instances_dir_path()
                .await
                .expect("delete settings"),
            PathBuf::from("/tmp/rumary-instances")
        );

        let instance = repo
            .create_instance(NewInstance {
                icon: String::new().try_into().expect("icon"),
                dir_name: format!("instance-{suffix}").try_into().expect("dir name"),
                display_name: "Test instance".to_owned().try_into().expect("display name"),
                version: "1.20.1".to_owned().try_into().expect("version"),
                description: "Repository integration test"
                    .to_owned()
                    .try_into()
                    .expect("description"),
                loader: Loader::Fabric("0.16.0".to_owned().try_into().expect("loader version")),
                is_public: true,
            })
            .await
            .expect("create instance");
        let instance_id = instance.id;
        assert!(instance.is_public);
        assert!(matches!(instance.loader, Loader::Fabric(_)));

        let updated_instance = repo
            .update_instance(
                instance_id,
                UpdateInstance {
                    icon: None,
                    dir_name: None,
                    display_name: Some(
                        "Updated instance"
                            .to_owned()
                            .try_into()
                            .expect("display name"),
                    ),
                    version: None,
                    description: None,
                    loader: Some(Loader::Vanilla),
                },
            )
            .await
            .expect("update instance");
        assert!(updated_instance.is_public);
        assert!(matches!(updated_instance.loader, Loader::Vanilla));
        assert_eq!(
            String::from(updated_instance.display_name),
            "Updated instance"
        );
        assert_eq!(
            repo.list_instances().await.expect("list instances").len(),
            1
        );

        let configuration = repo
            .create_config(NewConfiguration {
                icon: String::new().try_into().expect("icon"),
                dir_name: format!("config-{suffix}").try_into().expect("dir name"),
                display_name: "Test configuration"
                    .to_owned()
                    .try_into()
                    .expect("display name"),
                instance_id,
                is_public: true,
            })
            .await
            .expect("create configuration");
        let configuration_id = configuration.id;
        assert!(configuration.is_public);
        assert_eq!(configuration.instance_id, instance_id);

        let updated_configuration = repo
            .update_config(
                configuration_id,
                UpdateConfiguration {
                    icon: None,
                    dir_name: None,
                    display_name: Some(
                        "Updated configuration"
                            .to_owned()
                            .try_into()
                            .expect("display name"),
                    ),
                    instance_id: None,
                },
            )
            .await
            .expect("update configuration");
        assert_eq!(
            String::from(updated_configuration.display_name),
            "Updated configuration"
        );
        assert_eq!(
            repo.list_for_instance(instance_id)
                .await
                .expect("list configurations")
                .len(),
            1
        );

        repo.delete_config(configuration_id)
            .await
            .expect("delete configuration");
        repo.delete_instance(instance_id)
            .await
            .expect("delete instance");
        assert!(matches!(
            repo.get_instance(instance_id).await,
            Err(AppError::NotFound(_))
        ));
    }
}
