use sha1::Sha1;
use sha2::Sha256;
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
use sha2::Digest;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::task;

pub fn t(trans: &Translator, key: &str) -> String {
    trans.t(key)
}

pub fn assets_json_path<P: AsRef<Path>>(
    root_path: P,
    version: &str,
    asset_index: &str,
) -> PathBuf {
    root_path
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

pub fn minecraft_jar_path<P: AsRef<Path>>(root_path: P, version: &str) -> PathBuf {
    root_path
        .as_ref()
        .join("versions")
        .join(version)
        .join("client.jar")
}

pub fn objects_path<P: AsRef<Path>>(root_path: P, version: &str) -> PathBuf {
    root_path
        .as_ref()
        .join("assets")
        .join(version)
        .join("objects")
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

pub enum HashAlgo {
    Sha1,
    Sha256,
}

pub async fn string_to_hash(string: &str) -> UtilResult<Vec<u8>> {
    match hex::decode(string) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(Box::new(e)),
    }
}



pub async fn verify_file_hash<P, H>(
    path: P,
    expected_hash: H,
    algo: HashAlgo
) -> UtilResult<bool>
where
    P: AsRef<Path>,
    H: ToString,
{
    let hash = string_to_hash(&expected_hash.to_string()).await?;

    if !path.as_ref().exists() {
        return Ok(false);
    }

    match algo {
        HashAlgo::Sha1 => process_hash::<Sha1, _, _>(path, hash).await,
        HashAlgo::Sha256 => process_hash::<Sha256, _, _>(path, hash).await,
    }
}

async fn process_hash<D, P, H>(path: P, hash: H) -> UtilResult<bool>
where
    D: Digest + Default,
    P: AsRef<Path>,
    H: AsRef<[u8]>,
{
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut hasher = D::new();
    let mut buffer = [0; 8192];

    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let actual_hash = hasher.finalize();

    Ok(actual_hash.as_slice() == hash.as_ref())
}