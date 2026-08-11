use crate::domain::api::value_object::user::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BanScope {
    Account,
    Api,
    Launcher,
    Game,
}

impl BanScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Api => "api",
            Self::Launcher => "launcher",
            Self::Game => "game",
        }
    }

    pub fn blocks_api(self) -> bool {
        matches!(self, Self::Account | Self::Api)
    }
}

impl TryFrom<&str> for BanScope {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "account" => Ok(Self::Account),
            "api" => Ok(Self::Api),
            "launcher" => Ok(Self::Launcher),
            "game" => Ok(Self::Game),
            other => Err(format!("unknown ban scope: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BanId(pub Uuid);

impl From<Uuid> for BanId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<BanId> for Uuid {
    fn from(value: BanId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
pub struct UserBan {
    pub id: BanId,
    pub user_id: UserId,
    pub scope: BanScope,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason_code: String,
    pub staff_note: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub revoked_by: Option<UserId>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
}

impl UserBan {
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.starts_at <= now
            && self.expires_at.is_none_or(|expires_at| expires_at > now)
            && self.revoked_at.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct NewUserBan {
    pub user_id: UserId,
    pub scope: BanScope,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason_code: String,
    pub staff_note: Option<String>,
    pub created_by: UserId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn ban(starts_at: DateTime<Utc>, expires_at: Option<DateTime<Utc>>) -> UserBan {
        let user_id = UserId::from(Uuid::new_v4());
        UserBan {
            id: BanId::from(Uuid::new_v4()),
            user_id,
            scope: BanScope::Account,
            starts_at,
            expires_at,
            reason_code: "test".to_owned(),
            staff_note: None,
            created_by: user_id,
            created_at: starts_at,
            revoked_by: None,
            revoked_at: None,
            revoke_reason: None,
        }
    }

    #[test]
    fn active_window_is_checked() {
        let now = Utc::now();
        assert!(ban(now - Duration::minutes(1), None).is_active_at(now));
        assert!(!ban(now + Duration::minutes(1), None).is_active_at(now));
        assert!(!ban(now - Duration::minutes(2), Some(now)).is_active_at(now));
    }
}
