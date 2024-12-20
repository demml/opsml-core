use opsml_types::{SqlType, StorageType};
use pyo3::prelude::*;
use rand::Rng;
use std::default::Default;
use std::env;
use std::path::PathBuf;

/// ApiSettings for use with ApiClient
#[derive(Debug, Clone)]
#[pyclass]
pub struct ApiSettings {
    pub base_url: String,
    pub use_auth: bool,
    pub opsml_dir: String,
    pub scouter_dir: String,
    pub username: String,
    pub password: String,
    pub auth_token: String,
    pub prod_token: String,
}

/// StorageSettings for used with all storage clients
#[derive(Debug, Clone)]
#[pyclass]
pub struct OpsmlStorageSettings {
    pub storage_uri: String,
    pub client_mode: bool,
    pub api_settings: ApiSettings,
    pub storage_type: StorageType,
}

/// DatabaseSettings for used with all database clients
#[derive(Debug, Clone)]
#[pyclass]
pub struct OpsmlDatabaseSettings {
    pub connection_uri: String,
    pub max_connections: u32,
    pub sql_type: SqlType,
}

#[derive(Debug, Clone, Default)]
pub struct OpsmlAuthSettings {
    pub enabled: bool,
}

/// OpsmlConfig for use with both server and client implementations
/// OpsmlConfig is the main primary configuration struct for the Opsml system
/// Based on provided env variables, it will be used to determine if opsml is running in client or server mode.
#[derive(Debug, Clone)]
#[pyclass]
pub struct OpsmlConfig {
    pub app_name: String,
    pub app_env: String,
    pub app_version: String,
    pub opsml_storage_uri: String,
    pub opsml_tracking_uri: String,
    pub opsml_max_pool_connections: u32,
    pub opsml_prod_token: String,
    pub opsml_proxy_root: String,
    pub opsml_registry_path: String,
    pub opsml_testing: bool,
    pub download_chunk_size: usize,
    pub upload_chunk_size: usize,
    pub opsml_jwt_secret: String,
    pub opsml_refresh_secret: String,
    pub opsml_username: Option<String>,
    pub opsml_password: Option<String>,
    pub scouter_server_uri: Option<String>,
    pub scouter_username: Option<String>,
    pub scouter_password: Option<String>,
    pub scouter_auth: bool,
    pub opsml_auth: bool,
    pub client_mode: bool,
}

impl Default for OpsmlConfig {
    fn default() -> Self {
        let opsml_storage_uri =
            env::var("OPSML_STORAGE_URI").unwrap_or_else(|_| "./opsml_registries".to_string());

        let opsml_tracking_uri = env::var("OPSML_TRACKING_URI").unwrap_or_else(|_| {
            let mut current_dir = env::current_dir().expect("Failed to get current directory");
            current_dir.push("opsml.db");
            format!(
                "sqlite://{}",
                current_dir
                    .to_str()
                    .expect("Failed to convert path to string")
            )
        });

        let using_client = OpsmlConfig::is_using_client(&opsml_tracking_uri);

        OpsmlConfig {
            app_name: "opsml".to_string(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            opsml_storage_uri: OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri, using_client),
            opsml_tracking_uri,
            opsml_prod_token: env::var("OPSML_PROD_TOKEN").unwrap_or_else(|_| "".to_string()),

            opsml_proxy_root: "opsml-root:/".to_string(),
            opsml_registry_path: env::var("OPSML_REGISTRY_PATH")
                .unwrap_or_else(|_| "model_registry".to_string()),

            opsml_testing: env::var("OPSML_TESTING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            download_chunk_size: 31457280,
            upload_chunk_size: 31457280,

            opsml_jwt_secret: env::var("OPSML_JWT_SECRET")
                .unwrap_or_else(|_| generate_jwt_secret()),
            opsml_refresh_secret: env::var("OPSML_REFRESH_SECRET")
                .unwrap_or_else(|_| generate_jwt_secret()),

            opsml_username: env::var("OPSML_USERNAME").ok(),
            opsml_password: env::var("OPSML_PASSWORD").ok(),
            opsml_max_pool_connections: env::var("OPSML_MAX_POOL_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            scouter_server_uri: env::var("SCOUTER_SERVER_URI").ok(),
            scouter_username: env::var("SCOUTER_USERNAME").ok(),
            scouter_password: env::var("SCOUTER_PASSWORD").ok(),
            scouter_auth: env::var("SCOUTER_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            opsml_auth: env::var("OPSML_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            client_mode: using_client,
        }
    }
}

fn generate_jwt_secret() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

impl OpsmlConfig {
    pub fn set_opsml_storage_uri(opsml_storage_uri: String, using_client: bool) -> String {
        if using_client {
            return opsml_storage_uri;
        }

        if opsml_storage_uri.starts_with("gs://")
            || opsml_storage_uri.starts_with("s3://")
            || opsml_storage_uri.starts_with("az://")
        {
            opsml_storage_uri
        } else {
            let path = PathBuf::from(opsml_storage_uri);

            // check if the path exists, if not create it
            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap();
            }

            path.canonicalize().unwrap().to_str().unwrap().to_string()
        }
    }

    pub fn is_using_client(opsml_tracking_uri: &str) -> bool {
        opsml_tracking_uri.to_lowercase().trim().starts_with("http")
    }

    pub fn storage_root(&self) -> String {
        if !self.client_mode {
            let storage_uri_lower = self.opsml_storage_uri.to_lowercase();
            if storage_uri_lower.starts_with("gs://") {
                // strip the gs:// prefix
                storage_uri_lower.strip_prefix("gs://").unwrap().to_string()
            } else if storage_uri_lower.starts_with("s3://") {
                // strip the s3:// prefix
                storage_uri_lower.strip_prefix("s3://").unwrap().to_string()
            } else if storage_uri_lower.starts_with("az://") {
                // strip the az:// prefix
                storage_uri_lower.strip_prefix("az://").unwrap().to_string()
            } else {
                storage_uri_lower
            }
        } else {
            self.opsml_proxy_root.clone()
        }
    }

    pub fn auth_settings(&self) -> OpsmlAuthSettings {
        OpsmlAuthSettings {
            enabled: self.opsml_auth,
        }
    }

    fn get_storage_type(&self) -> StorageType {
        let storage_uri_lower = self.opsml_storage_uri.to_lowercase();
        if storage_uri_lower.starts_with("gs://") {
            StorageType::Google
        } else if storage_uri_lower.starts_with("s3://") {
            StorageType::AWS
        } else if storage_uri_lower.starts_with("az://") {
            StorageType::Azure
        } else {
            StorageType::Local
        }
    }

    fn get_sql_type(&self) -> SqlType {
        let tracking_uri_lower = self.opsml_tracking_uri.to_lowercase();
        if tracking_uri_lower.starts_with("postgres") {
            SqlType::Postgres
        } else if tracking_uri_lower.starts_with("mysql") {
            SqlType::MySql
        } else {
            SqlType::Sqlite
        }
    }
}

#[pymethods]
impl OpsmlConfig {
    /// Create a new OpsmlConfig instance
    ///
    /// # Returns
    ///
    /// `OpsmlConfig`: A new instance of OpsmlConfig
    #[new]
    #[pyo3(signature = (client_mode=None))]
    pub fn new(client_mode: Option<bool>) -> Self {
        let mut config = OpsmlConfig::default();

        if let Some(client_mode) = client_mode {
            config.client_mode = client_mode;
        }

        config
    }

    /// Get the storage settings for the OpsmlConfig
    pub fn storage_settings(&self) -> OpsmlStorageSettings {
        OpsmlStorageSettings {
            storage_uri: self.opsml_storage_uri.clone(),
            client_mode: self.client_mode,
            storage_type: self.get_storage_type(),
            api_settings: ApiSettings {
                base_url: self.opsml_tracking_uri.clone(),
                use_auth: self.opsml_auth,
                opsml_dir: "opsml".to_string(),
                scouter_dir: "scouter".to_string(),
                username: self.opsml_username.clone().unwrap_or_default(),
                password: self.opsml_password.clone().unwrap_or_default(),
                auth_token: "".to_string(),
                prod_token: self.opsml_prod_token.clone(),
            },
        }
    }

    pub fn database_settings(&self) -> OpsmlDatabaseSettings {
        let sql_type = self.get_sql_type();
        OpsmlDatabaseSettings {
            connection_uri: self.opsml_tracking_uri.clone(),
            max_connections: self.opsml_max_pool_connections,
            sql_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn cleanup() {
        // remove the directory
        // silently ignore errors
        std::fs::remove_dir_all("./test-bucket").ok();
        std::fs::remove_dir_all("./opsml_registries").ok();
    }

    #[test]
    fn test_generate_jwt_secret() {
        let jwt_secret = generate_jwt_secret();
        assert_eq!(jwt_secret.len(), 32);
    }

    #[test]
    fn test_set_opsml_storage_uri() {
        let opsml_storage_uri = "gs://test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri, false);
        assert_eq!(result, "gs://test-bucket");

        let opsml_storage_uri = "s3://test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri, false);
        assert_eq!(result, "s3://test-bucket");

        let opsml_storage_uri = "az://test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri, false);
        assert_eq!(result, "az://test-bucket");

        let opsml_storage_uri = "./test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri, false);
        assert_eq!(
            result,
            Path::new("./test-bucket")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );

        // remove the directory
        std::fs::remove_dir_all("./test-bucket").unwrap();
    }

    #[test]
    fn test_is_tracking_local() {
        let opsml_config = OpsmlConfig {
            opsml_tracking_uri: "sqlite:///opsml.db".to_string(),
            ..Default::default()
        };
        assert!(opsml_config.client_mode);

        let opsml_config = OpsmlConfig {
            opsml_tracking_uri: "http://localhost:5000".to_string(),
            ..Default::default()
        };
        assert!(!opsml_config.client_mode);

        cleanup();
    }

    #[test]
    fn test_storage_root() {
        let opsml_config = OpsmlConfig {
            opsml_storage_uri: "gs://test-bucket".to_string(),
            ..Default::default()
        };
        assert_eq!(opsml_config.storage_root(), "test-bucket");

        let opsml_config = OpsmlConfig {
            opsml_storage_uri: "s3://test-bucket".to_string(),
            ..Default::default()
        };
        assert_eq!(opsml_config.storage_root(), "test-bucket");

        let opsml_config = OpsmlConfig {
            opsml_storage_uri: "az://test-bucket".to_string(),
            ..Default::default()
        };

        assert_eq!(opsml_config.storage_root(), "test-bucket");

        let opsml_config = OpsmlConfig {
            opsml_storage_uri: "./test-bucket".to_string(),
            ..Default::default()
        };

        assert_eq!(opsml_config.storage_root(), "./test-bucket");

        let opsml_config = OpsmlConfig {
            opsml_tracking_uri: "http://localhost:5000".to_string(),
            opsml_storage_uri: "gs://test-bucket".to_string(),
            opsml_proxy_root: "opsml-root:/".to_string(),
            ..Default::default()
        };

        assert_eq!(opsml_config.storage_root(), "opsml-root:/");

        let opsml_config = OpsmlConfig {
            opsml_tracking_uri: "http://localhost:5000".to_string(),
            opsml_storage_uri: "s3://test-bucket".to_string(),
            opsml_proxy_root: "opsml-root:/".to_string(),
            ..Default::default()
        };

        assert_eq!(opsml_config.storage_root(), "opsml-root:/");
        cleanup();
    }

    #[test]
    fn test_auth_settings() {
        let opsml_config = OpsmlConfig {
            opsml_auth: true,
            ..Default::default()
        };
        let auth_settings = opsml_config.auth_settings();
        assert!(auth_settings.enabled);

        let opsml_config = OpsmlConfig {
            opsml_auth: false,
            ..Default::default()
        };
        let auth_settings = opsml_config.auth_settings();
        assert!(!auth_settings.enabled);
        cleanup();
    }

    #[test]
    fn test_default() {
        let opsml_config = OpsmlConfig::default();
        assert_eq!(opsml_config.app_name, "opsml");
        assert_eq!(opsml_config.app_env, "development");
        assert_eq!(opsml_config.app_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            opsml_config.opsml_storage_uri,
            Path::new("./opsml_registries")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(opsml_config.opsml_tracking_uri, "sqlite:///opsml.db");
        assert_eq!(opsml_config.opsml_prod_token, "staging");
        assert_eq!(opsml_config.opsml_proxy_root, "opsml-root:/");
        assert_eq!(opsml_config.opsml_registry_path, "model_registry");
        assert!(!opsml_config.opsml_testing);
        assert_eq!(opsml_config.download_chunk_size, 31457280);
        assert_eq!(opsml_config.upload_chunk_size, 31457280);
        assert_eq!(opsml_config.opsml_jwt_secret.len(), 32);
        assert_eq!(opsml_config.opsml_username, None);
        assert_eq!(opsml_config.opsml_password, None);
        assert_eq!(opsml_config.scouter_server_uri, None);
        assert_eq!(opsml_config.scouter_username, None);
        assert_eq!(opsml_config.scouter_password, None);
        assert!(!opsml_config.scouter_auth);
        assert!(!opsml_config.opsml_auth);

        cleanup();
    }
}
