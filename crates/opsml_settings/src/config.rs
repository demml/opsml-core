use pyo3::prelude::*;
use rand::Rng;
use std::default::Default;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct OpsmlAuthSettings {
    pub opsml_auth: bool,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct OpsmlConfig {
    pub app_name: String,
    pub app_env: String,
    pub app_version: String,
    pub opsml_storage_uri: String,
    pub opsml_tracking_uri: String,
    pub opsml_prod_token: String,
    pub opsml_proxy_root: String,
    pub opsml_registry_path: String,
    pub opsml_client_path_prefix: String,
    pub opsml_testing: bool,
    pub download_chunk_size: usize,
    pub upload_chunk_size: usize,
    pub opsml_jwt_secret: String,
    pub opsml_jwt_algorithm: String,
    pub opsml_username: Option<String>,
    pub opsml_password: Option<String>,
    pub scouter_server_uri: Option<String>,
    pub scouter_username: Option<String>,
    pub scouter_password: Option<String>,
    pub scouter_path_prefix: String,
    pub scouter_auth: bool,
    pub opsml_auth: bool,
}

impl Default for OpsmlConfig {
    fn default() -> Self {
        let opsml_storage_uri =
            env::var("OPSML_STORAGE_URI").unwrap_or_else(|_| "./opsml_registries".to_string());

        OpsmlConfig {
            app_name: "opsml".to_string(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            opsml_storage_uri: OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri),
            opsml_tracking_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "sqlite:///opsml.db".to_string()),
            opsml_prod_token: env::var("OPSML_PROD_TOKEN")
                .unwrap_or_else(|_| "staging".to_string()),

            opsml_proxy_root: "opsml-root:/".to_string(),
            opsml_registry_path: env::var("OPSML_REGISTRY_PATH")
                .unwrap_or_else(|_| "model_registry".to_string()),

            opsml_client_path_prefix: env::var("OPSML_CLIENT_PATH_PREFIX")
                .unwrap_or_else(|_| "opsml".to_string()),

            opsml_testing: env::var("OPSML_TESTING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            download_chunk_size: 31457280,
            upload_chunk_size: 31457280,

            opsml_jwt_secret: env::var("OPSML_JWT_SECRET")
                .unwrap_or_else(|_| generate_jwt_secret()),
            opsml_jwt_algorithm: env::var("OPSML_JWT_ALGORITHM")
                .unwrap_or_else(|_| "HS256".to_string()),

            opsml_username: env::var("OPSML_USERNAME").ok(),
            opsml_password: env::var("OPSML_PASSWORD").ok(),
            scouter_server_uri: env::var("SCOUTER_SERVER_URI").ok(),
            scouter_username: env::var("SCOUTER_USERNAME").ok(),
            scouter_password: env::var("SCOUTER_PASSWORD").ok(),
            scouter_path_prefix: "scouter".to_string(),
            scouter_auth: env::var("SCOUTER_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            opsml_auth: env::var("OPSML_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
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
    pub fn set_opsml_storage_uri(opsml_storage_uri: String) -> String {
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

    pub fn is_tracking_local(&self) -> bool {
        !self
            .opsml_tracking_uri
            .to_lowercase()
            .trim()
            .starts_with("http")
    }

    pub fn storage_root(&self) -> String {
        if self.is_tracking_local() {
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
            opsml_auth: self.opsml_auth,
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
    fn new() -> Self {
        OpsmlConfig::default()
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
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri);
        assert_eq!(result, "gs://test-bucket");

        let opsml_storage_uri = "s3://test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri);
        assert_eq!(result, "s3://test-bucket");

        let opsml_storage_uri = "az://test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri);
        assert_eq!(result, "az://test-bucket");

        let opsml_storage_uri = "./test-bucket".to_string();
        let result = OpsmlConfig::set_opsml_storage_uri(opsml_storage_uri);
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
        assert!(opsml_config.is_tracking_local());

        let opsml_config = OpsmlConfig {
            opsml_tracking_uri: "http://localhost:5000".to_string(),
            ..Default::default()
        };
        assert!(!opsml_config.is_tracking_local());

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
        assert!(auth_settings.opsml_auth);

        let opsml_config = OpsmlConfig {
            opsml_auth: false,
            ..Default::default()
        };
        let auth_settings = opsml_config.auth_settings();
        assert!(!auth_settings.opsml_auth);
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
        assert_eq!(opsml_config.opsml_client_path_prefix, "opsml");
        assert!(!opsml_config.opsml_testing);
        assert_eq!(opsml_config.download_chunk_size, 31457280);
        assert_eq!(opsml_config.upload_chunk_size, 31457280);
        assert_eq!(opsml_config.opsml_jwt_algorithm, "HS256");
        assert_eq!(opsml_config.opsml_username, None);
        assert_eq!(opsml_config.opsml_password, None);
        assert_eq!(opsml_config.scouter_server_uri, None);
        assert_eq!(opsml_config.scouter_username, None);
        assert_eq!(opsml_config.scouter_password, None);
        assert_eq!(opsml_config.scouter_path_prefix, "scouter");
        assert!(!opsml_config.scouter_auth);
        assert!(!opsml_config.opsml_auth);

        cleanup();
    }
}