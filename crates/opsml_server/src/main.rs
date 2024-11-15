use crate::core::router::create_router;
use crate::core::setup::setup_components;
use crate::core::state::AppState;
use anyhow::Ok;
use axum::Router;
use std::sync::Arc;

mod core;

async fn create_app() -> Result<Router, anyhow::Error> {
    // setup components (config, logging, storage client)
    let (config, storage_client) = setup_components().await?;

    // Create shared state for the application
    let app_state = Arc::new(AppState {
        storage_client: Arc::new(storage_client),
        config: Arc::new(config),
    });

    // create the router
    let app = create_router(app_state).await;

    Ok(app)
}

#[tokio::main]
async fn main() {
    // build our application with routes
    let app = create_app().await.unwrap();

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
