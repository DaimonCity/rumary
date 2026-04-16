use std::error::Error;
use std::path::{Path, PathBuf};
use reqwest_middleware::{ClientWithMiddleware};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::models::{
    AssetJson, Library, Version, VersionJson,
    VersionManifest,
};
use crate::app::AppState;
use crate::app::save_version_json;
use crate::util;

impl AppState {
    pub fn download_manifest(&self) {
        let tx = self.channels.manifest.0.clone();
        self.rt.spawn(async move {
            let result = reqwest::Client::new()
                .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
                .send()
                .await;
            if let Ok(response) = result
                && let Ok(manifest) = response.json::<VersionManifest>().await
            {
                let _ = tx.send(manifest);
            }
        });
    }

    pub fn download_minecraft_version(&mut self, version_json: VersionJson) {
        let reqwest_client = self.reqwest_client.clone();

        self.status = util::t(&self.translator, "downloading_version");
        let tx = self.channels.status.0.clone();

        let client_path = self.config.client_path.clone();
        let libs_path = if let Some(s) = self.get_libraries_path() {
            PathBuf::from(s)
        } else {
            return;
        };

        let index = if let Some(i) = self.selected_version {
            i
        } else {
            return;
        };

        let selected_ver = self.selected_version_name();

        self.versions[index].version_json = Some(version_json.clone());

        self.rt.spawn(async move {
            let save = save_version_json(client_path.as_str(), selected_ver, &version_json);
            let _ = tokio::join!(save);

            let mc_task = download_minecraft_jar(&reqwest_client, &client_path, &version_json);

            let libs = version_json.libraries.clone();
            let libs_task = download_libs_task(&reqwest_client, libs_path, libs);

            let assets_task = download_assets_json(&reqwest_client, &client_path, &version_json);

            let (mc_res, lib_res, assets_res) = tokio::join!(mc_task, libs_task, assets_task);

            if let Err(e) = mc_res {
                eprintln!("minecraft error: {e}");
            }
            if let Err(e) = lib_res {
                eprintln!("libs error: {e}");
            }
            if let Err(e) = assets_res {
                eprintln!("assets error: {e}");
            }

            let assets_task = download_assets(&reqwest_client, &client_path, version_json).await;
            if let Err(e) = assets_task {
                eprintln!("assets error: {e}");
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

async fn download_libs_task(
    client: &ClientWithMiddleware,
    root_lib_path: PathBuf,
    libs: Vec<Library>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for lib in libs {
        let artifact = lib.downloads.unwrap().artifact.unwrap();
        let url = artifact.url;
        let lib_path = artifact.path.unwrap();

        let bytes = util::download(client, &url).await?;

        let full_path = root_lib_path.join(&lib_path);

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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = &version_json.downloads.client.url;
    let bytes = util::download(client, url).await?;

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

async fn download_assets_json(
    client: &ClientWithMiddleware,
    client_path: &str,
    version_json: &VersionJson,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = &version_json.asset_index.url;
    let bytes = util::download(client, url).await?;

    let local_path = Path::new(&client_path)
        .join("assets")
        .join(&version_json.id)
        .join("indexes");

    if !local_path.exists() {
        fs::create_dir_all(&local_path).await?;
    }
    let file_path = local_path.join(format!("{}.json", version_json.asset_index.id));

    let mut file = File::create(&file_path).await?;
    file.write_all(&bytes).await?;
    println!("assets_json finished");
    Ok(())
}

async fn download_assets(
    client: &ClientWithMiddleware,
    client_path: &str,
    version_json: VersionJson,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let local_path = Path::new(&client_path)
        .join("assets")
        .join(&version_json.id);

    println!("{:?}", local_path);

    let file_path = local_path
        .join("indexes")
        .join(format!("{}.json", version_json.asset_index.id));
    let bytes = tokio::fs::read(file_path).await?;

    let local_path = local_path.join("objects");
    println!("{:?}", local_path);

    let json: AssetJson = serde_json::from_slice(&bytes)?;
    for res in json.objects {
        let res = res.1;
        let dir_name = &res.hash[0..2];
        let url = "https://resources.download.minecraft.net/".to_string() + dir_name + "/" + &*res.hash;
        let bytes = util::download(client, &url).await?;
        let dir_path = Path::new(&local_path).join(dir_name);

        if !dir_path.exists() {
            let _ = fs::create_dir_all(&dir_path).await;
        }

        let mut file = match File::create(dir_path.join(&res.hash)).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("assets error: {e}");
                return Err(Box::from(e));
            }
        };

        file.write_all(&bytes).await?;
    }

    println!("assets finished");
    Ok(())
}
