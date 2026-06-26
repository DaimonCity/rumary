use crate::download::{
    download_asset, download_assets_json, download_lib, download_minecraft_jar, get_assets_json,
    get_version_json,
};
use crate::result::ValidationResult;
use crate::util;
use crate::util::HashAlgo;
use reqwest::IntoUrl;
use reqwest_middleware::ClientWithMiddleware;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;
use rumary_dto::mojang::dto::response::{AssetJson, LibraryDownloads, VersionJson};

pub struct ValidationService {
    pub version_json: Arc<VersionJson>,
    assets_json: Arc<AssetJson>,
    reqwest_client: Arc<ClientWithMiddleware>,
    root_path: Arc<PathBuf>,
}

impl ValidationService {
    pub async fn new<U: IntoUrl, P: AsRef<Path>>(
        client: &ClientWithMiddleware,
        url: U,
        root_path: P,
    ) -> ValidationResult<Self> {
        let version_json = get_version_json(client, url).await?;
        let assets_json = get_assets_json(client, &version_json.asset_index.url).await?;

        Ok(Self {
            version_json: Arc::new(version_json),
            assets_json: Arc::new(assets_json),
            reqwest_client: Arc::new(client.clone()),
            root_path: Arc::new(root_path.as_ref().to_path_buf()),
        })
    }

    pub async fn validate_version(&self) -> ValidationResult<bool> {
        let client_task = self.validate_client();
        let assets_json_task = self.validate_assets_json();
        let assets_task = self.validate_assets();
        let libs_task = self.validate_libs();

        let (client_res, assets_json_res, assets_res, libs_res) =
            tokio::join!(client_task, assets_json_task, assets_task, libs_task);

        if let Err(e) = client_res {
            eprintln!("client_res error: {}", e);
            return Err(e);
        }

        if let Err(e) = assets_json_res {
            eprintln!("assets_json_res error: {}", e);
            return Err(e);
        }

        if let Err(e) = assets_res {
            eprintln!("assets_res error: {}", e);
            return Err(e);
        }

        if let Err(e) = libs_res {
            eprintln!("libs_res error: {}", e);
            return Err(e);
        }

        Ok(true)
    }

    async fn validate_libs(&self) -> ValidationResult<()> {
        let version_json = self.version_json.clone();
        let libraries = version_json.libraries.clone();

        let root_path = self.root_path.clone().deref().to_owned();
        let libs_path = util::get_libraries_path(&root_path);

        let mut set = JoinSet::new();


        for library in libraries {
            let artifacts = match library.downloads {
                None => {
                    vec!()
                }
                Some(lib) => {
                    match lib {
                        LibraryDownloads::Artifact(artifact) => {
                            vec!(artifact.unwrap())
                        }
                        LibraryDownloads::Classifiers(classifiers) => {
                            classifiers.unwrap().values().cloned().collect()
                        }
                    }
                }
            };

            for artifact in artifacts {
                let client = self.reqwest_client.clone();

                let url = artifact.url;

                let lib_path = artifact.path.unwrap();
                let file_path = libs_path.join(&lib_path);

                let hash = artifact.sha1;

                set.spawn(async move {
                    if !util::verify_file_hash(&file_path, &hash, HashAlgo::Sha1)
                        .await
                        .unwrap_or(false)
                    {
                        download_lib(&client, &file_path, &url).await?;
                    }
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                });
                if set.len() > 20
                    && let Some(res) = set.join_next().await
                {
                    res??;
                }
            }


        }

        while let Some(res) = set.join_next().await {
            res??; // Распаковываем результат выполнения и возможную ошибку внутри
        }

        Ok(())
    }

    async fn validate_client(&self) -> ValidationResult<()> {
        let client = self.reqwest_client.clone();
        let version_json = self.version_json.clone();
        let root_path = self.root_path.clone();
        let id = version_json.id.clone();
        let minecraft_jar_path = util::minecraft_jar_path(root_path.deref(), &id);
        let sha1 = version_json.downloads.client.sha1.clone();

        if !util::verify_file_hash(minecraft_jar_path, sha1, HashAlgo::Sha1)
            .await
            .unwrap_or(false)
        {
            download_minecraft_jar(&client, &root_path.deref(), version_json.deref()).await?;
        }

        Ok(())
    }

    async fn validate_assets_json(&self) -> ValidationResult<()> {
        let client = self.reqwest_client.clone();
        let version_json = self.version_json.clone();
        let root_path = self.root_path.clone();
        let id = version_json.id.clone();
        let asset_index = version_json.asset_index.id.clone();
        let assets_json_path = util::assets_json_path(root_path.deref(), &id, &asset_index);
        let sha1 = version_json.asset_index.sha1.clone();

        if !util::verify_file_hash(&assets_json_path, sha1, HashAlgo::Sha1)
            .await
            .unwrap_or(false)
        {
            download_assets_json(&client, &assets_json_path, version_json.deref()).await?;
        }

        Ok(())
    }

    async fn validate_assets(&self) -> ValidationResult<()> {
        println!("Validating assets...");

        let client = self.reqwest_client.clone();

        let version_json = self.version_json.clone();
        let objects = &self.assets_json.objects;

        let root_path = self.root_path.clone();
        let assets_path = util::objects_path(root_path.as_ref(), version_json.id.deref());

        let mut set = JoinSet::new();

        for asset in objects {
            let root_path = root_path.clone();
            let dir_name = &asset.1.hash[0..2];
            let file_name = &asset.1.hash;
            let file_path = assets_path.as_path().join(dir_name).join(file_name);

            let hash = asset.1.hash.clone();
            let client = client.clone();
            let id = version_json.id.clone();

            println!("Validating asset {}", file_path.display());

            set.spawn(async move {
                let client = client.clone();
                let id = id.clone();
                let hash = hash.clone();

                if !util::verify_file_hash(&file_path, &hash, HashAlgo::Sha1)
                    .await
                    .unwrap_or(false)
                {
                    println!("Failed to validate asset {}", &file_path.display());
                    download_asset(client.deref(), root_path.as_ref(), &id, &hash).await?;

                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });
            if set.len() > 50
                && let Some(res) = set.join_next().await
            {
                res??;
            }
        }

        while let Some(res) = set.join_next().await {
            res??; // Распаковываем результат выполнения и возможную ошибку внутри
        }

        Ok(())
    }
}
