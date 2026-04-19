use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use reqwest::IntoUrl;
use reqwest_middleware::ClientWithMiddleware;
use crate::download::{download_assets_json, download_minecraft_jar, get_assets_json, get_version_json};
use crate::models::{AssetJson, VersionJson};
use crate::result::UtilResult;
use crate::util;
use crate::util::HashAlgo::Sha1;

struct ValidationService {
    version_json: Arc<VersionJson>,
    assets_json: Arc<AssetJson>,
    reqwest_client: Arc<ClientWithMiddleware>,
    root_path: Arc<PathBuf>,
}

impl ValidationService {
    async fn new<U: IntoUrl, P: AsRef<Path>>(client: &ClientWithMiddleware, url: U, root_path: P) -> UtilResult<Self> {
        let version_json = get_version_json(client, url).await?;
        let assets_json = get_assets_json(client, &version_json.asset_index.url).await?;

        Ok(Self {
            version_json: Arc::new(version_json),
            assets_json: Arc::new(assets_json),
            reqwest_client: Arc::new(client.clone()),
            root_path: Arc::new(root_path.as_ref().to_path_buf()),
        })
    }

    pub async fn validate_version(&self) -> UtilResult<()> {
        let client_task = self.validate_client();
        let assets_json_task = self.validate_assets_json();

        let (client_res, assets_json_res) = tokio::join!(client_task, assets_json_task);

        if let Err(e) = client_res {
           eprintln!("{}", e);
        }

        if let Err(e) = assets_json_res {
            eprintln!("{}", e);
        }

        Ok(())
    }

    pub async fn validate_libs() {}

    pub async fn validate_client(&self) -> UtilResult<()> {
        let version_json = self.version_json.clone();
        let root_path = self.root_path.clone();
        let id = version_json.id.clone();
        let minecraft_jar_path = util::minecraft_jar_path(root_path.deref(), &id);
        let sha1 = version_json.downloads.client.sha1.clone();

        if !util::verify_file_hash(minecraft_jar_path, sha1, Sha1).await.unwrap_or(false) {
            download_minecraft_jar(&self.reqwest_client, &root_path.deref(), version_json.deref()).await?;
        }

        Ok(())
    }

    pub async fn validate_assets_json(&self) -> UtilResult<()> {
        let version_json = self.version_json.clone();
        let root_path = self.root_path.clone();
        let id = version_json.id.clone();
        let asset_index = version_json.asset_index.id.clone();
        let assets_json_path = util::assets_json_path(root_path.deref(), &id, &asset_index);
        let sha1 = version_json.asset_index.sha1.clone();

        if !util::verify_file_hash(assets_json_path, sha1, Sha1).await.unwrap_or(false) {
            download_assets_json(&self.reqwest_client, &root_path.deref(), version_json.deref()).await?;
        }

        Ok(())
    }

    pub async fn validate_assets() {}
}



