use crate::error::{AppError, AppResult};
use crate::repo::repository::TotpRepository;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use rand::TryRng;
use rand::rngs::SysRng;
use rumary_dto::domain::api::{NewTotpUser, TotpUser};
use rumary_dto::dto::api::response::TotpSetupResponse;
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

#[derive(Clone)]
pub struct TotpService {
    repo: Arc<dyn TotpRepository<Error=AppError>>,
    secret_key: [u8; 32],
}

impl TotpService {
    pub fn new(repo: Arc<dyn TotpRepository<Error=AppError>>, secret_key: [u8; 32]) -> Self {
        Self { repo, secret_key }
    }

    pub async fn enable_for_user(&self, user_uuid: Uuid) -> AppResult<TotpSetupResponse> {
        let secret = Secret::generate_secret().to_string();
        let (encrypted_secret, nonce) = encrypt(secret.as_bytes(), self.secret_key)?;

        let new_totp_user = NewTotpUser {
            uuid: user_uuid,
            encrypted_secret,
            nonce,
        };

        self.repo.create_totp_user(new_totp_user).await?; // By default, status will be NOT confirmed

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.into_bytes(),
            None,
            "rumary-test".to_string(),
        )
        .map_err(|_| AppError::Crypto("failed to create totp".to_string()))?;

        Ok(TotpSetupResponse {
            otp_auth_url: totp.get_url(),
        })
    }

    pub async fn confirm_for_user(&self, user_uuid: Uuid, code: &str) -> AppResult<()> {
        let user = self
            .repo
            .find_totp_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "totp user not found in confirming".to_string(),
            ))?;

        let res = self.verify_user_code(&user, code)?;

        if res {
            self.repo.totp_user_confirmed(user.uuid).await?;
        } else {
            self.repo.delete_totp_user(user.uuid).await?;
        }
        Ok(())
    }

    pub async fn delete_for_user(&self, user_uuid: Uuid, code: &str) -> AppResult<()> {
        let user = self
            .repo
            .find_totp_user(user_uuid)
            .await?
            .ok_or(AppError::NotFound(
                "totp user not found in deleting".to_string(),
            ))?;

        let res = self.verify_user_code(&user, code)?;
        if res {
            self.repo.delete_totp_user(user.uuid).await?;
            Ok(())
        } else {
            Err(AppError::NotFound(
                "totp user not found in deleting".to_string(),
            ))
        }
    }

    //関数（引数）　ー＞　戻り値
    pub fn verify_user_code(&self, user: &TotpUser, code: &str) -> AppResult<bool> {
        let encrypted_secret = user.totp.clone();
        let nonce = user.nonce.clone();

        let decrypted_secret = decrypt(&encrypted_secret, &nonce, self.secret_key)?;
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            decrypted_secret,
            None,
            "UCafe".to_string(),
        )
        .map_err(|_| AppError::Crypto("failed to create totp".to_string()))?;

        let is_valid = totp
            .check_current(code)
            .map_err(|_| AppError::Unauthorized("invalid totp code".to_string()))?;
        if !is_valid {
            return Err(AppError::Unauthorized("invalid totp code".to_string()));
        }

        Ok(true)
    }
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

    let arr_nonce= Nonce::from_iter(nonce);
    
    cipher
        .decrypt(&arr_nonce, ciphertext.as_slice())
        .map_err(|_| AppError::Crypto("failed to decrypt totp secret".to_string()))
}
