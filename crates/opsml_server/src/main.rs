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
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
    };
    use serde_qs;

    use axum::response::Response;
    use http_body_util::BodyExt; // for `collect`
    use opsml_settings::config::OpsmlDatabaseSettings;
    use opsml_sql::base::SqlClient;
    use opsml_sql::enums::client::SqlClientEnum;
    use opsml_types::{
        JwtToken, RegistryType, RepositoryRequest, RepositoryResponse, SqlType, UidRequest,
        UidResponse,
    };
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

    pub struct TestHelper {
        app: Router,
        token: JwtToken,
    }

    impl TestHelper {
        pub async fn new() -> Self {
            // set OPSML_AUTH to true
            env::set_var("OPSML_AUTH", "true");

            cleanup();

            // create the app
            let app = create_app().await.unwrap();

            // populate db
            setup().await;

            // retrieve the token
            let token = TestHelper::login(&app).await;

            Self { app, token }
        }

        pub async fn login(app: &Router) -> JwtToken {
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

            token
        }

        pub fn with_auth_header(&self, mut request: Request<Body>) -> Request<Body> {
            request.headers_mut().insert(
                header::AUTHORIZATION,
                format!("Bearer {}", self.token.token).parse().unwrap(),
            );

            request
        }

        pub async fn send_oneshot(&self, request: Request<Body>, use_auth: bool) -> Response<Body> {
            if use_auth {
                self.app
                    .clone()
                    .oneshot(self.with_auth_header(request))
                    .await
                    .unwrap()
            } else {
                self.app.clone().oneshot(request).await.unwrap()
            }
        }

        pub fn cleanup(&self) {
            cleanup();
        }
    }

    #[tokio::test]
    async fn test_opsml_server_login() {
        let helper = TestHelper::new().await;

        let request = Request::builder()
            .uri("/opsml/healthcheck")
            .body(Body::empty())
            .unwrap();

        let response = helper.send_oneshot(request, true).await;
        assert_eq!(response.status(), StatusCode::OK);

        // add invalid token
        let request = Request::builder()
            .uri("/opsml/healthcheck")
            .header(header::AUTHORIZATION, format!("Bearer {}", "invalid_token"))
            .body(Body::empty())
            .unwrap();

        // false will use the invalid token
        let response = helper.send_oneshot(request, false).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // refresh token
        let request = Request::builder()
            .uri("/opsml/auth/api/refresh")
            .body(Body::empty())
            .unwrap();

        let response = helper.send_oneshot(request, true).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let new_token: JwtToken = serde_json::from_slice(&body).unwrap();

        // check if the new token is different from the old token
        assert_ne!(helper.token.token, new_token.token);

        helper.cleanup();
    }

    #[tokio::test]
    async fn test_opsml_server_card_routes() {
        let helper = TestHelper::new().await;

        /////////////////////// Test check uid ///////////////////////

        // Test if a card UID exists - should be false
        let params = UidRequest {
            uid: "test_uid".to_string(),
            registry_type: RegistryType::Data,
        };

        let query_string = serde_qs::to_string(&params).unwrap();

        // check if a card UID exists (get request with UidRequest params)
        let request = Request::builder()
            .uri(format!("/opsml/card?{}", query_string))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = helper.send_oneshot(request, true).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let uid_response: UidResponse = serde_json::from_slice(&body).unwrap();

        // assert false
        assert_eq!(uid_response.exists, false);

        // Test if a card UID exists - should be True
        let params = UidRequest {
            uid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            registry_type: RegistryType::Data,
        };

        let query_string = serde_qs::to_string(&params).unwrap();

        // check if a card UID exists (get request with UidRequest params)
        let request = Request::builder()
            .uri(format!("/opsml/card?{}", query_string))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = helper.send_oneshot(request, true).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let uid_response: UidResponse = serde_json::from_slice(&body).unwrap();

        // assert true
        assert_eq!(uid_response.exists, true);

        /////////////////////// Test respositories ///////////////////////
        let params = RepositoryRequest {
            registry_type: RegistryType::Model,
        };

        let query_string = serde_qs::to_string(&params).unwrap();

        // check if a card UID exists (get request with UidRequest params)
        let request = Request::builder()
            .uri(format!("/opsml/card/repositories?{}", query_string))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = helper.send_oneshot(request, true).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let repository_response: RepositoryResponse = serde_json::from_slice(&body).unwrap();

        // assert 10
        assert_eq!(repository_response.repositories.len(), 10);

        helper.cleanup();
    }
}
