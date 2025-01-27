use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::OpenApi,
};
use axum::Extension;
use library::docs::docs_routes;
use state::{AppState, Repositories};
use domain::{
  health::health_routes,
  wormhole::{scan_routes, spy_routes}
};
use tokio::net::TcpListener;
use crate::storage::{Repository, memory::MemoryRepository};
use library::config::get_config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod domain;
pub mod library;
pub mod state;
pub mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,tower_http=debug".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    aide::generate::on_error(|error| {
        println!("{error}");
    });

    aide::generate::extract_schemas(true);

    let wormhole_repository = Repository::new(MemoryRepository::new());
    let vaas_repository = Repository::new(MemoryRepository::new());
    let repositories = Repositories::new(wormhole_repository, vaas_repository);
    let state = AppState::new(repositories).await?;

    let mut api = OpenApi::default();

    let app = ApiRouter::new()
        .merge(health_routes(Arc::new(state.clone())))
        .merge(scan_routes(Arc::new(state.clone())))
        .merge(spy_routes(Arc::new(state.clone())))
        .nest_api_service("/docs", docs_routes(Arc::new(state)))
        .finish_api_with(&mut api, library::docs::configure_api_docs)
        .layer(Extension(Arc::new(api)))
        .with_state(());

    let config = get_config();
    
    println!("Example docs are accessible at http://{}:{}/docs", config.host, config.port);

    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}