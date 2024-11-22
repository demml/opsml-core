use crate::core::router::create_router;
use crate::core::setup::setup_components;
use crate::core::state::AppState;
use anyhow::Ok;
use axum::Router;
use opsml_utils::color::LogColors;
use std::sync::Arc;
use tracing::info;

mod core;

async fn create_app() -> Result<Router, anyhow::Error> {
    // setup components (config, logging, storage client)
    let (config, storage_client) = setup_components().await?;

    // Create shared state for the application
    let app_state = Arc::new(AppState {
        storage_client: Arc::new(storage_client),
        config: Arc::new(config),
    });

    info!("Application state created");

    // create the router
    let app = create_router(app_state).await;

    info!("Router created");

    Ok(app)
}

#[tokio::main]
async fn main() {
    let logo = r#"
     ____             __  _____       _____                          
    / __ \____  _____/  |/  / /      / ___/___  ______   _____  _____
   / / / / __ \/ ___/ /|_/ / /       \__ \/ _ \/ ___/ | / / _ \/ ___/
  / /_/ / /_/ (__  ) /  / / /___    ___/ /  __/ /   | |/ /  __/ /    
  \____/ .___/____/_/  /_/_____/   /____/\___/_/    |___/\___/_/     
      /_/                                                            
               
    "#;

    println!("{}", LogColors::green(logo));

    // build our application with routes
    let app = create_app().await.unwrap();

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    println!("🚀 Server Running 🚀");
    axum::serve(listener, app).await.unwrap();
}
