use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("invalid utf8 from process: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type BuilderResult<T> = Result<T, BuilderError>;
