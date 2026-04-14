mod app;
mod config;
mod i18n;
mod models;
mod ui;

use app::LauncherApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = LauncherApp::new()?;
    app.run()?;
    Ok(())
}
