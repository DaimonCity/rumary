use async_trait::async_trait;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Client, InstallationRequest, LauncherBuild, Profile, Session, User},
    repository::AppRepository,
};

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppRepository for PostgresRepository {
    async fn insert_user(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            "insert into users (id, username, password_hash, auth_source, banned, created_at)
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(sqlx::types::Json(&user.auth_source))
        .bind(user.banned)
        .bind(user.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_conflict("user"))?;
        Ok(())
    }

    async fn find_user_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query(
            "select id, username, password_hash, auth_source, banned, created_at
             from users where username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_user).transpose()
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> AppResult<Option<User>> {
        let row = sqlx::query(
            "select id, username, password_hash, auth_source, banned, created_at
             from users where id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_user).transpose()
    }

    async fn update_user_ban(&self, user_id: Uuid, banned: bool) -> AppResult<User> {
        sqlx::query("update users set banned = $1 where id = $2")
            .bind(banned)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        self.find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user `{user_id}`")))
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        let rows = sqlx::query(
            "select id, username, password_hash, auth_source, banned, created_at
             from users order by created_at desc",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_user).collect()
    }

    async fn insert_session(&self, session: &Session) -> AppResult<()> {
        sqlx::query("insert into sessions (token, user_id, issued_at) values ($1, $2, $3)")
            .bind(&session.token)
            .bind(session.user_id)
            .bind(session.issued_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_client(&self, client: &Client) -> AppResult<()> {
        sqlx::query(
            "insert into clients
             (id, slug, display_name, minecraft_version, authlib_injector_url, files, rules, launch_arguments, created_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(client.id)
        .bind(&client.slug)
        .bind(&client.display_name)
        .bind(&client.minecraft_version)
        .bind(&client.authlib_injector_url)
        .bind(json(&client.files)?)
        .bind(json(&client.rules)?)
        .bind(json(&client.launch_arguments)?)
        .bind(client.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_conflict("client"))?;
        Ok(())
    }

    async fn list_clients(&self) -> AppResult<Vec<Client>> {
        let rows = sqlx::query(
            "select id, slug, display_name, minecraft_version, authlib_injector_url, files, rules, launch_arguments, created_at
             from clients order by created_at desc",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_client).collect()
    }

    async fn find_client_by_id(&self, client_id: Uuid) -> AppResult<Option<Client>> {
        let row = sqlx::query(
            "select id, slug, display_name, minecraft_version, authlib_injector_url, files, rules, launch_arguments, created_at
             from clients where id = $1",
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_client).transpose()
    }

    async fn insert_profile(&self, profile: &Profile) -> AppResult<()> {
        sqlx::query(
            "insert into profiles (id, client_id, slug, display_name, mods, rules, created_at)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(profile.id)
        .bind(profile.client_id)
        .bind(&profile.slug)
        .bind(&profile.display_name)
        .bind(json(&profile.mods)?)
        .bind(json(&profile.rules)?)
        .bind(profile.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_conflict("profile"))?;
        Ok(())
    }

    async fn find_profile_by_id(&self, profile_id: Uuid) -> AppResult<Option<Profile>> {
        let row = sqlx::query(
            "select id, client_id, slug, display_name, mods, rules, created_at
             from profiles where id = $1",
        )
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_profile).transpose()
    }

    async fn insert_launcher_build(&self, build: &LauncherBuild) -> AppResult<()> {
        sqlx::query(
            "insert into launcher_builds (id, version, channel, download_url, checksum, changelog, published_at)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(build.id)
        .bind(build.version.to_string())
        .bind(&build.channel)
        .bind(&build.download_url)
        .bind(&build.checksum)
        .bind(&build.changelog)
        .bind(build.published_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn latest_launcher_build(&self, channel: &str) -> AppResult<Option<LauncherBuild>> {
        let row = sqlx::query(
            "select id, version, channel, download_url, checksum, changelog, published_at
             from launcher_builds where channel = $1 order by published_at desc",
        )
        .bind(channel)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_launcher_build).transpose()
    }

    async fn insert_installation_request(&self, request: &InstallationRequest) -> AppResult<()> {
        sqlx::query(
            "insert into installation_requests
             (id, user_id, client_id, profile_id, platform, launcher_version, created_at)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(request.id)
        .bind(request.user_id)
        .bind(request.client_id)
        .bind(request.profile_id)
        .bind(&request.platform)
        .bind(request.launcher_version.as_ref().map(ToString::to_string))
        .bind(request.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_user(row: sqlx::postgres::PgRow) -> AppResult<User> {
    Ok(User {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        password_hash: row.try_get("password_hash")?,
        auth_source: from_json(row.try_get("auth_source")?)?,
        banned: row.try_get("banned")?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_client(row: sqlx::postgres::PgRow) -> AppResult<Client> {
    Ok(Client {
        id: row.try_get("id")?,
        slug: row.try_get("slug")?,
        display_name: row.try_get("display_name")?,
        minecraft_version: row.try_get("minecraft_version")?,
        authlib_injector_url: row.try_get("authlib_injector_url")?,
        files: from_json(row.try_get("files")?)?,
        rules: from_json(row.try_get("rules")?)?,
        launch_arguments: from_json(row.try_get("launch_arguments")?)?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_profile(row: sqlx::postgres::PgRow) -> AppResult<Profile> {
    Ok(Profile {
        id: row.try_get("id")?,
        client_id: row.try_get("client_id")?,
        slug: row.try_get("slug")?,
        display_name: row.try_get("display_name")?,
        mods: from_json(row.try_get("mods")?)?,
        rules: from_json(row.try_get("rules")?)?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_launcher_build(row: sqlx::postgres::PgRow) -> AppResult<LauncherBuild> {
    let version: String = row.try_get("version")?;
    Ok(LauncherBuild {
        id: row.try_get("id")?,
        version: Version::parse(&version).map_err(|err| AppError::Database(err.to_string()))?,
        channel: row.try_get("channel")?,
        download_url: row.try_get("download_url")?,
        checksum: row.try_get("checksum")?,
        changelog: row.try_get("changelog")?,
        published_at: row.try_get("published_at")?,
    })
}

fn json<T: Serialize>(value: &T) -> AppResult<sqlx::types::Json<serde_json::Value>> {
    serde_json::to_value(value)
        .map(sqlx::types::Json)
        .map_err(|err| AppError::Internal(err.to_string()))
}

fn from_json<T: DeserializeOwned>(value: sqlx::types::Json<serde_json::Value>) -> AppResult<T> {
    serde_json::from_value(value.0).map_err(|err| AppError::Database(err.to_string()))
}

fn map_conflict(entity: &'static str) -> impl Fn(sqlx::Error) -> AppError {
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
