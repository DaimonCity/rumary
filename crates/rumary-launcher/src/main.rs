mod app;
mod config;
mod i18n;
mod models;
mod ui;
pub mod download;
pub mod util;
pub mod result;
pub mod validation;

use app::LauncherApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = LauncherApp::new()?;
    app.run()?;
    Ok(())
}
