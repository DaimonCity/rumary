use crate::app::AppState;
use crate::models::{AssetJson, Library, Version, VersionJson, VersionManifest};
use crate::result::UtilResult;
use crate::util;
use reqwest::IntoUrl;
use reqwest_middleware::ClientWithMiddleware;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::task::JoinSet;

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

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

        let root_path = self.config.root_path.clone();
        let Some(ver) = self.get_selected_version_name() else {
            return;
        };

        let libs_path = util::get_libraries_path(&root_path, &ver);
        drop(ver);

        self.rt.spawn(async move {
            let version_json_path = util::version_json_path(root_path.as_str(), &version_json.id);

            let libs = version_json.libraries.clone();

            let assets_task = download_assets(&reqwest_client, &root_path, &version_json);
            let libs_task = download_libs_task(&reqwest_client, libs_path, libs);
            let mc_task = download_minecraft_jar(&reqwest_client, &root_path, &version_json);
            let version_json_save_task = util::save_json(version_json_path, version_json.clone());

            let (mc_res, lib_res, assets_res, version_json_save_res) =
                tokio::join!(mc_task, libs_task, assets_task, version_json_save_task);

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
        let client = self.reqwest_client.clone();

        let f = async move {
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

pub async fn get_version_json<U: IntoUrl>(
    client: &ClientWithMiddleware,
    url: U,
) -> UtilResult<VersionJson> {
    let response = util::get_response(client, url).await?;
    let version_json: VersionJson = response.json().await?;
    Ok(version_json)
}

pub async fn get_assets_json<U: IntoUrl>(
    client: &ClientWithMiddleware,
    url: U,
) -> UtilResult<AssetJson> {
    let response = util::get_response(client, url).await?;
    let assets_json: AssetJson = response.json().await?;
    Ok(assets_json)
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
        let full_path = root_lib_path.as_ref().join(&lib_path);

        download_lib(client, &full_path, &url).await?;
    }
    println!("libs finished");

    Ok(())
}

pub async fn download_lib<U: IntoUrl, P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    lib_path: P,
    url: U,
) -> UtilResult<()> {
    let bytes = util::download_file(client, url).await?;
    util::save_file(lib_path, &bytes).await?;

    Ok(())
}

pub async fn download_minecraft_jar<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    client_path: P,
    version_json: &VersionJson,
) -> UtilResult<()> {
    let client = client.clone();
    let url = version_json.downloads.client.url.clone();

    let local_path = client_path.as_ref().join("versions").join(&version_json.id);

    let mut set = JoinSet::new();

    set.spawn(async move {
        if !local_path.exists() {
            fs::create_dir_all(&local_path).await?;
        }
        let bytes = util::download_file(&client, url).await?;
        let file_path = local_path.join("client.jar");
        util::save_file(file_path, &bytes).await?;
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    while let Some(res) = set.join_next().await {
        res??; // Распаковываем результат выполнения и возможную ошибку внутри
    }

    println!("minecraft jar finished");

    Ok(())
}

pub async fn download_assets_json<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    file_path: P,
    version_json: &VersionJson,
) -> UtilResult<()> {
    if let Some(parent) = file_path.as_ref().parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut set = JoinSet::new();
    let url = version_json.asset_index.url.clone();
    let client = client.clone();
    let file_path = file_path.as_ref().to_path_buf();

    set.spawn(async move {
        let bytes = util::download_file(&client, url).await?;
        let json: Arc<AssetJson> = Arc::new(serde_json::from_slice(&bytes)?);
        util::save_json(file_path, json).await?;
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    while let Some(res) = set.join_next().await {
        res??; // Распаковываем результат выполнения и возможную ошибку внутри
    }

    println!("assets_json finished");

    Ok(())
}

async fn download_assets<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    root_path: P,
    version_json: &VersionJson,
) -> UtilResult<()> {
    let path = util::assets_json_path(
        root_path.as_ref(),
        &version_json.id,
        &version_json.asset_index.id,
    );

    if !path.exists() {
        download_assets_json(client, &path, version_json).await?;
    }

    if !path.exists() {
        return Err("failed to download asset".into());
    }

    println!("assets_json finished");

    let json: AssetJson = util::read_json(path).await?;

    let mut set = JoinSet::new();

    for (_, res) in json.objects {
        let client = client.clone();

        let id = version_json.id.clone();
        let root_path = root_path.as_ref().to_path_buf();
        let hash = res.hash.clone();

        set.spawn(async move {
            download_asset(&client, root_path, &id, &hash).await?;
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });

        if set.len() > 20
            && let Some(res) = set.join_next().await
        {
            res??;
        }
    }

    while let Some(res) = set.join_next().await {
        res??; // Распаковываем результат выполнения и возможную ошибку внутри
    }

    println!("assets finished");
    Ok(())
}

pub async fn download_asset<P: AsRef<Path>>(
    client: &ClientWithMiddleware,
    root_path: P,
    version: &str,
    hash: &str,
) -> UtilResult<()> {
    let client = client.clone();

    let dir_name = &hash[0..2];
    let local_path = util::objects_path(root_path, version);
    let file_path = local_path.join(dir_name).join(hash);

    let url = format!("{}/{}/{}", RESOURCES_URL, dir_name, hash);

    let bytes = util::download_file(&client, &url).await?;
    util::save_file(file_path, bytes.as_ref()).await?;

    Ok(())
}
