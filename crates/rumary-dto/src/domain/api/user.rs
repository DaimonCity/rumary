use crate::domain::api::{AccessLevel, RoleId};
use crate::domain::auth::tokens::{TokenHash, TokenId};
use crate::domain::user::{Login, Nickname, PasswordHash, UserId};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub login: Login,
    pub nickname: Nickname,
    pub password_hash: PasswordHash,
    pub access_level: AccessLevel,
    pub roles: Vec<RoleId>,
    pub ban: Ban,
}

pub struct UserSession {
    pub id: UserId,
    pub token_id: TokenId,
    pub refresh_token_hash: TokenHash,
    pub expires_at: DateTime<Utc>,
}

pub struct NewUser {
    pub login: Login,
    pub nickname: Nickname,
    pub password_hash: PasswordHash,
}
#[derive(Debug, Clone)]
pub struct TotpUser {
    pub id: UserId,
    pub totp: String,
    pub nonce: String,
    pub confirmed: bool,
}

pub struct NewTotpUser {
    pub user_id: Uuid,
    pub encrypted_secret: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ban(u8);

impl Default for Ban {
    fn default() -> Self {
        Self::new()
    }
}

impl Ban {
    // Определяем константы для каждого бита (1, 2, 4, 8, 16...)
    const IP: u8 = 0b0000_0001; // 1
    const HWID: u8 = 0b0000_0010; // 2
    const ACCOUNT: u8 = 0b0000_0100; // 4

    pub fn new() -> Self {
        Self(0)
    }

    // Установить бан
    pub fn set_ip(&mut self, enabled: bool) {
        if enabled {
            self.0 |= Self::IP;
        } else {
            self.0 &= !Self::IP;
        }
    }

    pub fn set_hwid(&mut self, enabled: bool) {
        if enabled {
            self.0 |= Self::HWID;
        } else {
            self.0 &= !Self::HWID;
        }
    }

    pub fn set_account(&mut self, enabled: bool) {
        if enabled {
            self.0 |= Self::ACCOUNT; // 0b0000_0000 | 0b0000_0100 => 0b0000_0100
        } else {
            self.0 &= !Self::ACCOUNT; // 0b0000_0000 & 0b1111_1011 => 0b0000_0100
        }
    }

    // Проверка
    pub fn is_ip_banned(&self) -> bool {
        (self.0 & Self::IP) != 0
    }
    pub fn is_hwid_banned(&self) -> bool {
        (self.0 & Self::HWID) != 0
    }
    pub fn is_account_banned(&self) -> bool {
        (self.0 & Self::ACCOUNT) != 0
    }

    // Комбинированная проверка (например, IP или HWID)
    pub fn is_any_banned(&self) -> bool {
        self.0 != 0
    }
}
