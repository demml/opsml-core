use crate::core::router::create_router;
use crate::core::setup::setup_components;
use crate::core::state::AppState;
use anyhow::Ok;
use anyhow::Result;
use axum::Router;
use opsml_auth::auth::AuthManager;
use opsml_utils::color::LogColors;
use std::sync::Arc;
use tracing::{info, warn};

mod core;

async fn create_app() -> Result<Router> {
    // setup components (config, logging, storage client)
    let (config, storage_client, sql_client) = setup_components().await?;
    let auth_enabled = config.opsml_auth;

    // Create shared state for the application (storage client, auth manager, config)
    let app_state = Arc::new(AppState {
        storage_client: Arc::new(storage_client),
        sql_client: Arc::new(sql_client),
        auth_manager: Arc::new(AuthManager::new(
            &config.opsml_jwt_secret,
            &config.opsml_refresh_secret,
        )),
        config: Arc::new(config),
    });

    info!("✅ Application state created");

    // create the router
    let app = create_router(app_state).await?;

    info!("✅ Router created");

    if auth_enabled {
        info!("✅ Auth enabled");
    } else {
        warn!("Auth disabled");
    }

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

    // get OPSML_SERVER_PORT from env
    let port = std::env::var("OPSML_SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    println!("🚀 Server Running 🚀");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::health::schema::Alive;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
    };

    use http_body_util::BodyExt; // for `collect`
    use opsml_settings::config::OpsmlDatabaseSettings;
    use opsml_settings::config::SqlType;
    use opsml_sql::base::SqlClient;
    use opsml_sql::enums::client::SqlClientEnum;
    use opsml_types::types::JwtToken;
    use std::env;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    fn cleanup() {
        // cleanup delete opsml.db and opsml_registries folder from the current directory
        let current_dir = std::env::current_dir().unwrap();
        let db_path = current_dir.join("opsml.db");
        let registry_path = current_dir.join("opsml_registries");

        if db_path.exists() {
            std::fs::remove_file(db_path).unwrap();
        }

        if registry_path.exists() {
            std::fs::remove_dir_all(registry_path).unwrap();
        }
    }

    fn get_connection_uri() -> String {
        let mut current_dir = env::current_dir().expect("Failed to get current directory");
        current_dir.push("opsml.db");

        format!(
            "sqlite://{}",
            current_dir
                .to_str()
                .expect("Failed to convert path to string")
        )
    }

    async fn setup() {
        let config = OpsmlDatabaseSettings {
            connection_uri: get_connection_uri(),
            max_connections: 1,
            sql_type: SqlType::Sqlite,
        };

        let client = SqlClientEnum::new(&config).await.unwrap();

        // Run the SQL script to populate the database
        let script = std::fs::read_to_string("tests/populate_db.sql").unwrap();
        client.query(&script).await;
    }

    #[tokio::test]
    async fn test_opsml_server_healthcheck() {
        cleanup();

        let app = create_app().await.unwrap();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/opsml/healthcheck")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        // check if Alive
        let response: Alive = serde_json::from_slice(&body).unwrap();

        assert_eq!(response.status, "Alive");

        cleanup();
    }

    #[tokio::test]
    async fn test_opsml_server_login() {
        // set OPSML_AUTH to true
        env::set_var("OPSML_AUTH", "true");

        cleanup();

        let app = create_app().await.unwrap();

        // setup the database
        setup().await;

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        // create header map
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/opsml/auth/api/login")
                    .header("Username", "admin")
                    .header("Password", "test_password")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let token: JwtToken = serde_json::from_slice(&body).unwrap();

        // call the healthcheck endpoint with the token
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/opsml/healthcheck")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/opsml/healthcheck")
                    .header(header::AUTHORIZATION, format!("Bearer {}", "invalid_token"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // refresh token
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/opsml/auth/api/refresh")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let new_token: JwtToken = serde_json::from_slice(&body).unwrap();

        // check if the new token is different from the old token
        assert_ne!(token.token, new_token.token);

        cleanup();
    }
}
