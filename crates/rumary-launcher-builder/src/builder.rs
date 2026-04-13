use std::path::{Path, PathBuf};

use chrono::Utc;
use semver::Version;
use tokio::{fs, process::Command};

use crate::{
    api_client::{PublishReleaseRequest, RumaryApiClient},
    error::{BuilderError, BuilderResult},
};

#[derive(Debug, Clone)]
pub struct BuildLauncherRequest {
    pub repository_url: String,
    pub git_ref: String,
    pub workspace_dir: PathBuf,
    pub artifact_name: String,
    pub channel: String,
    pub version: Version,
    pub changelog: Option<String>,
    pub publish_download_url: String,
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub artifact_path: PathBuf,
    pub built_at: chrono::DateTime<Utc>,
}

pub struct GithubLauncherBuilder {
    api_client: RumaryApiClient,
}

impl GithubLauncherBuilder {
    pub fn new(api_client: RumaryApiClient) -> Self {
        Self { api_client }
    }

    pub async fn build_and_publish(
        &self,
        request: &BuildLauncherRequest,
    ) -> BuilderResult<BuildResult> {
        let repo_dir = request.workspace_dir.join("launcher-src");
        self.prepare_repository(&request.repository_url, &request.git_ref, &repo_dir)
            .await?;
        self.build_repository(&repo_dir).await?;

        let artifact_path = repo_dir
            .join("target")
            .join("release")
            .join(&request.artifact_name);
        if fs::metadata(&artifact_path).await.is_err() {
            return Err(BuilderError::CommandFailed(format!(
                "artifact `{}` was not produced by cargo build --release",
                artifact_path.display()
            )));
        }

        self.api_client
            .publish_launcher_release(&PublishReleaseRequest {
                version: request.version.clone(),
                channel: request.channel.clone(),
                download_url: request.publish_download_url.clone(),
                checksum: None,
                changelog: request.changelog.clone(),
            })
            .await?;

        Ok(BuildResult {
            artifact_path,
            built_at: Utc::now(),
        })
    }

    async fn prepare_repository(
        &self,
        repository_url: &str,
        git_ref: &str,
        repo_dir: &Path,
    ) -> BuilderResult<()> {
        if fs::metadata(repo_dir).await.is_err() {
            run(Command::new("git")
                .arg("clone")
                .arg(repository_url)
                .arg(repo_dir))
            .await?;
        } else {
            run(Command::new("git")
                .current_dir(repo_dir)
                .arg("fetch")
                .arg("--all")
                .arg("--tags"))
            .await?;
        }

        run(Command::new("git")
            .current_dir(repo_dir)
            .arg("checkout")
            .arg(git_ref))
        .await?;
        run(Command::new("git")
            .current_dir(repo_dir)
            .arg("pull")
            .arg("--ff-only"))
        .await?;
        Ok(())
    }

    async fn build_repository(&self, repo_dir: &Path) -> BuilderResult<()> {
        run(Command::new("cargo")
            .current_dir(repo_dir)
            .arg("build")
            .arg("--release"))
        .await
    }
}

async fn run(command: &mut Command) -> BuilderResult<()> {
    let output = command.output().await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8(output.stderr)?;
    Err(BuilderError::CommandFailed(stderr))
}
