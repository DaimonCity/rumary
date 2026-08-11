use crate::domain::api::{BanScope, UserBan};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UserBanResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub scope: BanScope,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason_code: String,
    pub staff_note: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub revoked_by: Option<Uuid>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
}

impl From<UserBan> for UserBanResponse {
    fn from(value: UserBan) -> Self {
        Self {
            id: value.id.into(),
            user_id: value.user_id.into(),
            scope: value.scope,
            starts_at: value.starts_at,
            expires_at: value.expires_at,
            reason_code: value.reason_code,
            staff_note: value.staff_note,
            created_by: value.created_by.into(),
            created_at: value.created_at,
            revoked_by: value.revoked_by.map(Into::into),
            revoked_at: value.revoked_at,
            revoke_reason: value.revoke_reason,
        }
    }
}
