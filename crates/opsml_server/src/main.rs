use axum::{response::Html, routing::get, Router};
use opsml_storage::core::storage::base::FileSystem;
use opsml_storage::core::storage::google::google_storage::GCSFSStorageClient;

#[tokio::main]
async fn main() {
    // build our application with routes
    let app = Router::new()
        .route("/", get(handler))
        .route("/health", get(health_check));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn health_check() -> &'static str {
    GCSFSStorageClient::new("opsml-storage-integration".to_string());
    "OK"
}
