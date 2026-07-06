use crate::app::Application;
use crate::config::Config;
use crate::error::AppResult;
use tracing::log;

mod app;
mod config;
mod error;
mod repo;
mod service;
mod services;
mod state;
mod util;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    if let Err(err) = run().await {
        log::error!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let config = Config::from_env()?;
    let app = Application::build(config).await?;
    app.run().await
}
