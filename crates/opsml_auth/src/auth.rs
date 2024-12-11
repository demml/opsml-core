use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use opsml_sql::enums::client::SqlClientEnum;
use opsml_sql::schemas::schema::User;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct OpsmlUser {
    pub user: User, // this comes from the db
}

impl AuthUser for OpsmlUser {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.user.id.unwrap()
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.user.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: SqlClientEnum,
}

impl Backend {
    pub fn new(db: SqlClientEnum) -> Self {
        Self { db }
    }
}
