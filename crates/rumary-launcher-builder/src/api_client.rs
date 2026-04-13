use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::BuilderResult;

#[derive(Clone)]
pub struct RumaryApiClient {
    base_url: Url,
    client: reqwest::Client,
}

impl RumaryApiClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn publish_launcher_release(
        &self,
        request: &PublishReleaseRequest,
    ) -> BuilderResult<()> {
        let url = self.base_url.join("api/launcher/releases")?;
        self.client
            .post(url)
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn latest_launcher_release(&self, channel: &str) -> BuilderResult<serde_json::Value> {
        let mut url = self.base_url.join("api/launcher/download/latest")?;
        url.query_pairs_mut().append_pair("channel", channel);
        let response = self.client.get(url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    pub async fn check_launcher_update(
        &self,
        channel: &str,
        current_version: &Version,
    ) -> BuilderResult<serde_json::Value> {
        let mut url = self.base_url.join("api/launcher/updates/check")?;
        url.query_pairs_mut()
            .append_pair("channel", channel)
            .append_pair("current_version", &current_version.to_string());
        let response = self.client.get(url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    pub async fn create_installation_plan(
        &self,
        request: &CreateInstallationRequest,
    ) -> BuilderResult<serde_json::Value> {
        let url = self.base_url.join("api/installations")?;
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishReleaseRequest {
    pub version: Version,
    pub channel: String,
    pub download_url: String,
    pub checksum: Option<String>,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstallationRequest {
    pub user_id: uuid::Uuid,
    pub client_id: uuid::Uuid,
    pub profile_id: Option<uuid::Uuid>,
    pub platform: String,
    pub launcher_version: Option<Version>,
}
