use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{PgPool, Row};

use crate::error::{AppError, AppResult};

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
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
