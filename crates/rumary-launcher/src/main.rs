mod app;
mod config;
pub mod download;
mod i18n;
pub mod result;
mod ui;
pub mod util;
pub mod validation;

use app::LauncherApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = LauncherApp::new().await?;
    app.run()?;
    Ok(())
}
