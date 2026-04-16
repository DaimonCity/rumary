use std::error::Error;
use std::path::{Path, PathBuf};
use reqwest_middleware::ClientWithMiddleware;
use bytes::Bytes;
use serde::Deserialize;
use tokio::io::{AsyncWriteExt, BufWriter};
use crate::i18n::Translator;
use crate::models::VersionJson;

pub fn t(trans: &Translator, key: &str) -> String {
    trans.t(key)
}

pub async fn download(client: &ClientWithMiddleware, url: &str) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
    Ok(client.get(url).send().await?.bytes().await?)
}

pub async fn save_file(file_path: PathBuf, bytes: Bytes) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let file = tokio::fs::File::create(file_path).await?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&bytes).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn save_json<'a, J: Deserialize<'a>> (
    json_path: &Path, bytes: &Bytes
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(parent) = json_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let file = tokio::fs::File::create(json_path).await?;
    let mut writer = BufWriter::new(file);
    writer.write_all(bytes).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn assets_json_is_valid(client_path: &str, version_json: &VersionJson) -> Result<PathBuf, PathBuf> {
    let local_path = Path::new(&client_path)
        .join("assets")
        .join(&version_json.id)
        .join("indexes");
    let file_path = local_path.join(format!("{}.json", version_json.asset_index.id));

    if file_path.exists() {
        Ok(file_path)
    } else {
        Err(file_path)
    }
}