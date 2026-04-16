use std::error::Error;
use reqwest_middleware::ClientWithMiddleware;
use bytes::Bytes;
use crate::i18n::Translator;

pub fn t(trans: &Translator, key: &str) -> String {
    trans.t(key)
}

pub async fn download(client: &ClientWithMiddleware, url: &str) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
    Ok(client.get(url).send().await?.bytes().await?)
}

