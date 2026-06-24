mod api;
mod config;
mod db;
mod error;
mod models;
mod repository;
mod services;
mod state;
pub mod builder;
pub mod util;

use std::sync::Arc;

use config::AppConfig;
use dotenvy::dotenv;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    api::build_router,
    db::PostgresRepository,
    services::{LocalAuthProvider, StubMinecraftProvider, StubSkinService},
    state::AppState,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rumary_api=debug,tower_http=debug".into()),
        )
        .init();

    let _ = dotenv();
    let config = AppConfig::from_env().expect("failed to read application configuration");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    Migrator::new(std::path::Path::new("./crates/rumary-api/migrations"))
        .await
        .expect("failed to load postgres migrations")
        .run(&pool)
        .await
        .expect("failed to run postgres migrations");

    let repository = Arc::new(PostgresRepository::new(pool));
    let state = Arc::new(AppState::new(
        repository,
        Arc::new(LocalAuthProvider),
        Arc::new(StubMinecraftProvider),
        Arc::new(StubSkinService::default()),
    ));

    let app = build_router(state);
    let listener = TcpListener::bind(config.bind_addr)
        .await
        .expect("failed to bind tcp listener");

    info!("backend listening on http://{}", config.bind_addr);

    axum::serve(listener, app)
        .await
        .expect("server terminated unexpectedly");
}
