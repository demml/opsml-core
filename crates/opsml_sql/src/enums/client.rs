use crate::base::SqlClient;
use crate::mysql::client::MySqlClient;
use crate::postgres::client::PostgresClient;
use crate::sqlite::client::SqliteClient;
use opsml_settings::config::{OpsmlDatabaseSettings, SqlType};

pub enum SqlClientEnum {
    Postgres(PostgresClient),
    Sqlite(SqliteClient),
    MySql(MySqlClient),
}

impl SqlClientEnum {
    pub async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        match settings.sql_type {
            SqlType::Postgres => {
                let client = PostgresClient::new(settings).await;
                SqlClientEnum::Postgres(client)
            }
            SqlType::Sqlite => {
                let client = SqliteClient::new(settings).await;
                SqlClientEnum::Sqlite(client)
            }
            SqlType::MySql => {
                let client = MySqlClient::new(settings).await;
                SqlClientEnum::MySql(client)
            }
        }
    }
}
