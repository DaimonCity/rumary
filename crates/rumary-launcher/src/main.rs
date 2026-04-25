mod app;
mod config;
mod i18n;
mod ui;
pub mod download;
pub mod util;
pub mod result;
pub mod validation;

use app::LauncherApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = LauncherApp::new().await?;
    app.run()?;
    Ok(())
}
