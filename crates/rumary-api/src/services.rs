
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use crate::{
    error::{AppError, AppResult},
};

#[derive(Default)]
pub struct LocalAuthProvider;

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Internal(format!("password hashing failed: {err}")))
}

fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|err| AppError::Internal(format!("stored password hash is invalid: {err}")))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))
}
