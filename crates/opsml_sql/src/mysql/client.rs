use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{
    mysql::{MySql, MySqlPoolOptions},
    Pool,
};

pub struct MySqlClient {
    pub pool: Pool<MySql>,
}

#[async_trait]
impl SqlClient for MySqlClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = MySqlPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to Postgres database");

        Self { pool }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_mysql() -> () {
        let config = OpsmlDatabaseSettings {
            connection_uri: "sqlite://:memory:".to_string(),
            max_connections: 1,
        };

        let client = MySqlClient::new(&config).await;
    }
}
