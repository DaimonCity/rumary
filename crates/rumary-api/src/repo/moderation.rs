use crate::error::{AppError, AppResult};
use crate::repo::db::PostgresRepo;
use crate::repo::repository::ModerationRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rumary_dto::domain::api::value_object::user::UserId;
use rumary_dto::domain::api::{BanId, BanScope, NewUserBan, UserBan};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct UserBanRow {
    id: Uuid,
    account_id: Uuid,
    scope: String,
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    reason_code: String,
    staff_note: Option<String>,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    revoked_by: Option<Uuid>,
    revoked_at: Option<DateTime<Utc>>,
    revoke_reason: Option<String>,
}

impl TryFrom<UserBanRow> for UserBan {
    type Error = AppError;

    fn try_from(row: UserBanRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: BanId::from(row.id),
            user_id: UserId::from(row.account_id),
            scope: BanScope::try_from(row.scope.as_str()).map_err(AppError::Internal)?,
            starts_at: row.starts_at,
            expires_at: row.expires_at,
            reason_code: row.reason_code,
            staff_note: row.staff_note,
            created_by: UserId::from(row.created_by),
            created_at: row.created_at,
            revoked_by: row.revoked_by.map(UserId::from),
            revoked_at: row.revoked_at,
            revoke_reason: row.revoke_reason,
        })
    }
}

#[async_trait]
impl ModerationRepository for PostgresRepo {
    type Error = AppError;

    async fn create_user_ban_and_revoke_sessions(&self, ban: NewUserBan) -> AppResult<UserBan> {
        let mut tx = self.pool_ref().begin().await?;
        let user_id = Uuid::from(ban.user_id);

        // Serializes moderation operations for one account without locking the user row.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
        if !user_exists {
            return Err(AppError::NotFound(format!("user {user_id}")));
        }

        let duplicate_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM moderation_bans
                WHERE subject_type = 'account'
                  AND account_id = $1
                  AND scope = $2
                  AND revoked_at IS NULL
                  AND starts_at <= now()
                  AND (expires_at IS NULL OR expires_at > now())
            )
            "#,
        )
        .bind(user_id)
        .bind(ban.scope.as_str())
        .fetch_one(&mut *tx)
        .await?;
        if duplicate_exists {
            return Err(AppError::Conflict(format!(
                "user already has an active {} ban",
                ban.scope.as_str()
            )));
        }

        let row: UserBanRow = sqlx::query_as(
            r#"
            INSERT INTO moderation_bans (
                subject_type, account_id, scope, starts_at, expires_at,
                reason_code, staff_note, created_by
            )
            VALUES ('account', $1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, account_id, scope, starts_at, expires_at, reason_code, staff_note,
                created_by, created_at, revoked_by, revoked_at, revoke_reason
            "#,
        )
        .bind(user_id)
        .bind(ban.scope.as_str())
        .bind(ban.starts_at)
        .bind(ban.expires_at)
        .bind(ban.reason_code)
        .bind(ban.staff_note)
        .bind(Uuid::from(ban.created_by))
        .fetch_one(&mut *tx)
        .await?;

        // Ban creation and revocation of both refresh and access tokens must be atomic.
        if ban.starts_at <= Utc::now() {
            sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                r#"
                UPDATE users
                SET token_version = token_version + 1,
                    updated_at = now()
                WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        row.try_into()
    }

    async fn find_active_api_ban(&self, user_id: UserId) -> AppResult<Option<UserBan>> {
        let row: Option<UserBanRow> = sqlx::query_as(
            r#"
            SELECT
                id, account_id, scope, starts_at, expires_at, reason_code, staff_note,
                created_by, created_at, revoked_by, revoked_at, revoke_reason
            FROM moderation_bans
            WHERE
            subject_type = 'account'
              AND account_id = $1
              AND scope IN ('account', 'api')
              AND revoked_at IS NULL
              AND starts_at <= now()
              AND (expires_at IS NULL OR expires_at > now())
            ORDER BY CASE scope WHEN 'account' THEN 0 ELSE 1 END, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(Uuid::from(user_id))
        .fetch_optional(self.pool_ref())
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_user_bans(&self, user_id: UserId) -> AppResult<Vec<UserBan>> {
        let rows: Vec<UserBanRow> = sqlx::query_as(
            r#"
            SELECT
                id, account_id, scope, starts_at, expires_at, reason_code, staff_note,
                created_by, created_at, revoked_by, revoked_at, revoke_reason
            FROM moderation_bans
            WHERE subject_type = 'account' AND account_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(Uuid::from(user_id))
        .fetch_all(self.pool_ref())
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn revoke_user_ban(
        &self,
        ban_id: BanId,
        user_id: UserId,
        revoked_by: UserId,
        reason: String,
    ) -> AppResult<Option<UserBan>> {
        let row: Option<UserBanRow> = sqlx::query_as(
            r#"
            UPDATE moderation_bans
            SET revoked_by = $3, revoked_at = now(), revoke_reason = $4
            WHERE id = $1
              AND subject_type = 'account'
              AND account_id = $2
              AND revoked_at IS NULL
            RETURNING
                id, account_id, scope, starts_at, expires_at, reason_code, staff_note,
                created_by, created_at, revoked_by, revoked_at, revoke_reason
            "#,
        )
        .bind(Uuid::from(ban_id))
        .bind(Uuid::from(user_id))
        .bind(Uuid::from(revoked_by))
        .bind(reason)
        .fetch_optional(self.pool_ref())
        .await?;

        row.map(TryInto::try_into).transpose()
    }
}
