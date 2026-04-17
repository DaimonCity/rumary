use std::error::Error;

pub type UtilResult<T> = Result<T, Box<dyn Error + Send + Sync>>;