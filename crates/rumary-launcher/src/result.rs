use std::error::Error;

pub type AppResult<T> = Result<T, Box<dyn Error>>;
pub type UtilResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
