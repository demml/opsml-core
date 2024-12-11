use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use opsml_sql::enums::client::SqlClientEnum;
use opsml_sql::schemas::schema::User;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
