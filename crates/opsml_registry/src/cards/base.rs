use opsml_error::error::CardError;
use opsml_types::*;
use std::collections::HashMap;
use std::env;

pub struct BaseArgs {
    pub name: String,
    pub repository: String,
    pub contact: String,
    pub version: String,
    pub uid: String,
    pub tags: HashMap<String, String>,
}

impl BaseArgs {
    pub fn new(
        name: Option<String>,
        repository: Option<String>,
        contact: Option<String>,
        version: Option<String>,
        uid: Option<String>,
        info: Option<CardInfo>,
        tags: HashMap<String, String>,
    ) -> Result<Self, CardError> {
        // check if name provided
        // if not provided, check in info is provided and if name is provided in info
        // if not provided, check if OPSML_RUNTIME_NAME is set
        // if not set, return error

        let name = name
            .or_else(|| info.and_then(|i| i.name))
            .or_else(|| env::var("OPSML_RUNTIME_NAME").ok())
            .ok_or_else(|| CardError::Error("Name not provided".to_string()))?;

        let repository = repository
            .or_else(|| info.and_then(|i| i.repository))
            .or_else(|| env::var("OPSML_RUNTIME_REPOSITORY").ok())
            .ok_or_else(|| CardError::Error("Repository not provided".to_string()))?;

        let contact = contact
            .or_else(|| info.and_then(|i| i.repository))
            .or_else(|| env::var("OPSML_RUNTIME_CONTACT").ok())
            .ok_or_else(|| CardError::Error("Contact not provided".to_string()))?;
        let version = version.unwrap_or(enums::CommonKwargs::BaseVersion.to_string());
        let uid = uid.unwrap_or(enums::CommonKwargs::Undefined.to_string());

        Ok(Self {
            name,
            repository,
            contact,
            version,
            uid,
            tags,
        })
    }
}
