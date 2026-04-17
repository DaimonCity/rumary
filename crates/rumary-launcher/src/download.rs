use crate::app::AppState;
use crate::models::{AssetJson, Library, Version, VersionJson, VersionManifest};
use crate::util;
use reqwest_middleware::ClientWithMiddleware;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;
use crate::result::UtilResult;

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

impl AppState {
    pub fn download_manifest(&self) {
        let tx = self.channels.manifest.0.clone();
        let reqwest_client = self.reqwest_client.clone();

        self.rt.spawn(async move {
            if let Ok(response) = util::get_response(&reqwest_client, MANIFEST_URL).await
                && let Ok(manifest) = response.json::<VersionManifest>().await
            {
                let _ = tx.send(manifest);
            }
        });
    }

    pub fn download_minecraft_version(&mut self, version_json: Arc<VersionJson>) {
        let reqwest_client = self.reqwest_client.clone();

        self.status = util::t(&self.translator, "downloading_version");
        let tx = self.channels.status.0.clone();

        let client_path = self.config.client_path.clone();
        let libs_path = if let Some(s) = self.get_libraries_path() {
            PathBuf::from(s)
        } else {
            return;
        };

        self.rt.spawn(async move {
            let version_json_path = util::version_json_path(client_path.as_str(), &version_json.id);

            let libs = version_json.libraries.clone();

            let assets_task = download_assets(&reqwest_client, &client_path, &version_json);
            let libs_task = download_libs_task(&reqwest_client, libs_path, libs);
            let mc_task = download_minecraft_jar(&reqwest_client, &client_path, &version_json);
            let version_json_save_task = util::save_json(version_json_path, version_json.clone());



            let (mc_res, lib_res, assets_res, version_json_save_res) = tokio::join!(mc_task, libs_task, assets_task, version_json_save_task);

            if let Err(e) = mc_res {
                eprintln!("minecraft_task  error: {e}");

            }
            if let Err(e) = lib_res {
                eprintln!("libs_task  error: {e}");
            }
            if let Err(e) = assets_res {

                eprintln!("assets_task error: {e}");
            }

            if let Err(e) = version_json_save_res {
                eprintln!("version_json_save_task error: {e}");
            }

            let _ = tx.send("Version downloaded!".to_string());
        });

    }

    pub fn fetch_download_version_json(&self, version: Version) {
        let tx = self.channels.minecraft.0.clone();

        let f = async move {
            let client = reqwest::Client::new();

            let response = client.get(&version.url).send().await;

            if let Ok(response) = response
                && let Ok(version_json) = response.json::<VersionJson>().await
            {
                let _ = tx.send(version_json);
            }
        };

        self.rt.spawn(f);
    }
}

async fn download_libs_task<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    root_lib_path: P,
    libs: Vec<Library>,
) -> UtilResult<()> {
    for lib in libs {
        let artifact = lib.downloads.unwrap().artifact.unwrap();
        let url = artifact.url;
        let lib_path = artifact.path.unwrap();

        let bytes = util::download_file(client, &url).await?;

        let full_path = root_lib_path.as_ref().join(&lib_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(full_path).await?;
        file.write_all(&bytes).await?;
    }
    println!("libs finished");

    Ok(())
}

async fn download_minecraft_jar(
    client: &ClientWithMiddleware,
    client_path: &str,
    version_json: &VersionJson,
) -> UtilResult<()> {
    let url = &version_json.downloads.client.url;
    let bytes = util::download_file(client, url).await?;

    let local_path = Path::new(&client_path)
        .join("versions")
        .join(&version_json.id);

    if !local_path.exists() {
        fs::create_dir_all(&local_path).await?;
    }

    let file_path = local_path.join("client.jar");

    let mut file = File::create(file_path).await?;
    file.write_all(&bytes).await?;
    println!("minecraft jar finished");

    Ok(())
}

async fn download_assets_json<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    file_path: P,
    version_json: &VersionJson,
) -> UtilResult<()> {
    if let Some(parent) = file_path.as_ref().parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let url = &version_json.asset_index.url;
    let bytes = util::download_file(client, url).await?;

    let json: Arc<AssetJson> = Arc::new(serde_json::from_slice(&bytes)?);

    util::save_json(file_path, json).await?;
    println!("assets_json finished");

    Ok(())
}

async fn download_assets<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    client_path: P,
    version_json: &VersionJson,
) -> UtilResult<()> {
    let path = util::asset_json_path(client_path.as_ref(), &version_json.id,  &version_json.asset_index.id);

    for _ in 0..3 {
        if path.exists() {
            break;
        }

        download_assets_json(client, &path, version_json).await?;
    }

    if !path.exists() {
        return Err("failed to download asset".into());
    }

    println!("assets_json finished");

    let json: AssetJson = util::read_json(path).await?;

    let local_path = client_path.as_ref()
        .join("assets")
        .join(&version_json.id)
        .join("objects");
    println!("{:?}", local_path);

    let mut set = JoinSet::new();

    for (_key, res) in json.objects {
        let dir_name = &res.hash[0..2];
        let url = format!("https://resources.download.minecraft.net/{}/{}", dir_name, res.hash);

        let file_path = local_path.join(dir_name).join(&res.hash);
        let client = client.clone();

        set.spawn(async move {
            let bytes = util::download_file(&client, &url).await?;
            util::save_file(file_path, bytes.as_ref()).await?;
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });

        if set.len() > 20 && let Some(res) = set.join_next().await {
            res??;
        }
    }

    while let Some(res) = set.join_next().await {
        res??; // Распаковываем результат выполнения и возможную ошибку внутри
    }

    println!("assets finished");
    Ok(())
}


