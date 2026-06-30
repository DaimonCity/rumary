use tracing::log;
use crate::app::Application;
use crate::config::Config;

mod config;
mod error;
mod state;
mod util;
mod app;
pub mod service;
pub mod repo;
pub mod services;

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
