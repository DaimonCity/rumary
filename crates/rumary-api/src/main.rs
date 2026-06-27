use tracing::log;
use crate::app::Application;
use crate::config::Config;

mod api;
mod config;
mod db;
mod error;
mod repository;
mod services;
mod state;
mod util;
mod auth;
mod totp;
mod app;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    if let Err(err) = run().await {
        log::error!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), error::AppError> {
    let config = Config::from_env()?;
    let app = Application::build(config).await?;
    app.run().await
}
