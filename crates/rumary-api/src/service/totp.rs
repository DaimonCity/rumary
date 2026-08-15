use crate::error::{AppError, AppResult};
use crate::repo::repository::TotpRepository;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use rand::TryRng;
use rand::rngs::SysRng;
use rumary_dto::domain::api::NewTotpUser;
use rumary_dto::domain::api::value_object::totp::TotpCode;
use rumary_dto::domain::api::value_object::user::UserId;
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, Totp};

#[derive(Clone)]
pub struct TotpService {
    repo: Arc<dyn TotpRepository<Error = AppError>>,
    secret_key: [u8; 32],
}

impl TotpService {
    pub fn new(repo: Arc<dyn TotpRepository<Error = AppError>>, secret_key: [u8; 32]) -> Self {
        Self { repo, secret_key }
    }

    pub async fn is_enabled(&self, user_id: UserId) -> AppResult<bool> {
        if let Some(t) = self.repo.find_totp_user(user_id).await? {
            return Ok(t.confirmed);
        }
        Ok(false)
    }

    pub async fn enable_for_user(&self, user_id: UserId) -> AppResult<Totp> /*TotpSetupResponse*/ {
        if self.is_enabled(user_id).await? {
            return Err(AppError::Conflict("2FA already enabled".to_string()));
        }
        let secret = Secret::generate();
        let (encrypted_secret, nonce) = encrypt(secret.as_bytes(), self.secret_key)?;

        let new_totp_user = NewTotpUser {
            user_id: user_id.into(),
            encrypted_secret,
            nonce,
        };

        self.repo.create_totp_user(new_totp_user).await?; // By default, status will be NOT confirmed

        let totp = totp(secret)?;

        Ok(totp)
    }

    pub async fn confirm_for_user(&self, user_id: UserId, code: TotpCode) -> AppResult<()> {
        let res = self.verify_user_code(user_id, code).await?;
        if res {
            self.repo.totp_user_enable(user_id).await?;
            Ok(())
        } else {
            self.repo.delete_totp_user(user_id).await?;
            Err(AppError::Unauthorized("invalid totp code".to_string()))
        }
    }

    pub async fn delete_for_user(&self, user_id: UserId, code: TotpCode) -> AppResult<()> {
        let res = self.verify_user_code(user_id, code).await?;
        if res {
            self.repo.delete_totp_user(user_id).await?;
            Ok(())
        } else {
            Err(AppError::NotFound(
                "totp user not found in deleting".to_string(),
            ))
        }
    }

    pub async fn delete_totp_user(&self, user_id: UserId) -> AppResult<()> {
        if self.is_enabled(user_id).await? {
            self.repo.delete_totp_user(user_id).await?;
        }
        Ok(())
    }

    //関数（引数）　ー＞　戻り値
    pub async fn verify_user_code(&self, user_id: UserId, code: TotpCode) -> AppResult<bool> {
        let user = self
            .repo
            .find_totp_user(user_id)
            .await?
            .ok_or(AppError::NotFound("totp user was not found".to_string()))?;

        let encrypted_secret = user.totp.clone();
        let nonce = user.nonce.clone();

        let decrypted_secret = decrypt(&encrypted_secret, &nonce, self.secret_key)?;
        let totp = totp(decrypted_secret)?;

        let step = if let Some(step) = totp.check_current(code.as_str()) {
            step as i64
        } else {
            return Ok(false);
        };

        let updated = self.repo.save_used_step_if_newer(user_id, step).await?;

        if !updated {
            return Ok(false);
        }

        Ok(true)
    }
}

fn totp(secret: impl Into<Secret>) -> AppResult<Totp> {
    totp_rs::Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_secret(secret)
        .with_account_name("rumary")
        .with_digits(6)
        .build()
        .map_err(Into::into)
}

fn encrypt(text: &[u8], key: [u8; 32]) -> AppResult<(String, String)> {
    let arr_key = Key::from(key);
    let cipher = ChaCha20Poly1305::new(&arr_key);
    let mut nonce = [0u8; 12];
    SysRng
        .try_fill_bytes(&mut nonce)
        .map_err(|_| AppError::Crypto("failed to create nonce".parse().unwrap()))?;

    let arr_nonce = Nonce::from(nonce);
    let ciphertext = cipher
        .encrypt(&arr_nonce, text)
        .map_err(|_| AppError::Crypto("failed to encrypt totp secret".parse().unwrap()))?;

    Ok((hex::encode(ciphertext), hex::encode(arr_nonce)))
}

fn decrypt(ciphertext: &str, nonce: &str, key: [u8; 32]) -> AppResult<Vec<u8>> {
    let arr_key = Key::from(key);

    let ciphertext = hex::decode(ciphertext)
        .map_err(|_| AppError::Crypto("invalid encrypted totp secret".to_string()))?;
    let nonce =
        hex::decode(nonce).map_err(|_| AppError::Crypto("invalid totp nonce".to_string()))?;
    let cipher = ChaCha20Poly1305::new(&arr_key);

    let arr_nonce = Nonce::from_iter(nonce);

    cipher
        .decrypt(&arr_nonce, ciphertext.as_slice())
        .map_err(|_| AppError::Crypto("failed to decrypt totp secret".to_string()))
}
