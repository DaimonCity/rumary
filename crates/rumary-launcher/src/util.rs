use crate::i18n::Translator;
use crate::result::UtilResult;
use bytes::Bytes;
use reqwest::{IntoUrl, Response};
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task;

pub fn t(trans: &Translator, key: &str) -> String {
    trans.t(key)
}

pub fn asset_json_path<P: AsRef<Path>>(
    client_path: P,
    version: &str,
    asset_index: &str,
) -> PathBuf {
    client_path
        .as_ref()
        .join("assets")
        .join(version)
        .join("indexes")
        .join(format!("{}.json", asset_index))
}

pub fn get_libraries_path<P: AsRef<Path>>(client_path: P, version: &str) -> PathBuf {
    client_path.as_ref().join("libraries").join(version)
}

pub fn version_json_path<P: AsRef<Path>>(client_path: P, version: &str) -> PathBuf {
    client_path
        .as_ref()
        .join("versions")
        .join(version)
        .join("version.json")
}

pub fn minecraft_jar_path<P: AsRef<Path>>(client_path: P, version: &str) -> PathBuf {
    client_path
        .as_ref()
        .join("versions")
        .join(version)
        .join("client.jar")
}

pub async fn download_file<U: IntoUrl>(client: &ClientWithMiddleware, url: U) -> UtilResult<Bytes> {
    Ok(get_response(client, url).await?.bytes().await?)
}

pub async fn get_response<U: IntoUrl>(
    client: &ClientWithMiddleware,
    url: U,
) -> UtilResult<Response> {
    Ok(client.get(url).send().await?)
}

pub async fn read_json<P: AsRef<Path>, J: DeserializeOwned + Send + 'static>(
    json_path: P,
) -> Result<J, Box<dyn Error + Send + Sync>> {
    if !json_path.as_ref().exists() {
        return Err(Box::from("json_path does not exist"));
    }

    let bytes = tokio::fs::read(json_path).await?;

    task::spawn_blocking(move || -> Result<J, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_slice(&bytes)?)
    })
    .await?
}

pub async fn save_file<P: AsRef<Path>>(file_path: P, bytes: &[u8]) -> UtilResult<()> {
    if let Some(parent) = file_path.as_ref().parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let file = tokio::fs::File::create(file_path).await?;
    let mut writer = BufWriter::new(file);
    writer.write_all(bytes).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn save_json<P: AsRef<Path>, J: Serialize + Send + Sync + 'static>(
    json_path: P,
    json: Arc<J>,
) -> UtilResult<()> {
    let data = task::spawn_blocking({
        let json = json.clone();
        move || serde_json::to_vec_pretty(json.deref())
    })
    .await??;

    save_file(json_path, &data).await?;

    Ok(())
}

pub async fn _save_json<P: AsRef<Path>, J: Serialize + Send + Sync + 'static>(
    json_path: P,
    json: J,
) -> UtilResult<()> {
    let json = Arc::new(json);
    let data = task::spawn_blocking({
        let json = json.clone();
        move || serde_json::to_vec_pretty(json.deref())
    })
    .await??;

    save_file(json_path, &data).await?;

    Ok(())
}
