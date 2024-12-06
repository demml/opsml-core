/// this file contains helper logic for generating sql queries across different databases
use crate::schemas::schema::{
    AuditCardRecord, DataCardRecord, ModelCardRecord, PipelineCardRecord, RunCardRecord,
};
use serde_json;
pub struct SqlHelper;

impl SqlHelper {
    pub fn get_datacard_insert_query(card: &DataCardRecord) -> String {
        let mut columns = vec![
            "uid",
            "date",
            "timestamp",
            "app_env",
            "name",
            "repository",
            "major",
            "minor",
            "patch",
            "contact",
            "data_type",
            "interface_type",
            "tags",
            "runcard_uid",
            "pipelinecard_uid",
            "auditcard_uid",
        ];
        let mut values = vec![
            format!("'{}'", card.uid),
            format!("'{}'", card.date),
            format!("{}", card.timestamp),
            format!("'{}'", card.app_env),
            format!("'{}'", card.name),
            format!("'{}'", card.repository),
            format!("{}", card.major),
            format!("{}", card.minor),
            format!("{}", card.patch),
            format!("'{}'", card.contact),
            format!("'{}'", card.data_type),
            format!("'{}'", card.interface_type),
            format!("'{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("'{}'", card.runcard_uid),
            format!("'{}'", card.pipelinecard_uid),
            format!("'{}'", card.auditcard_uid),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        format!(
            "INSERT INTO opsml_data_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }

    pub fn get_modelcard_insert_query(card: &ModelCardRecord) -> String {
        let mut columns = vec![
            "uid",
            "date",
            "timestamp",
            "app_env",
            "name",
            "repository",
            "major",
            "minor",
            "patch",
            "contact",
            "datacard_uid",
            "sample_data_type",
            "model_type",
            "interface_type",
            "task_type",
            "tags",
            "runcard_uid",
            "pipelinecard_uid",
            "auditcard_uid",
        ];
        let mut values = vec![
            format!("'{}'", card.uid),
            format!("'{}'", card.date),
            format!("{}", card.timestamp),
            format!("'{}'", card.app_env),
            format!("'{}'", card.name),
            format!("'{}'", card.repository),
            format!("{}", card.major),
            format!("{}", card.minor),
            format!("{}", card.patch),
            format!("'{}'", card.contact),
            format!("'{}'", card.datacard_uid),
            format!("'{}'", card.sample_data_type),
            format!("'{}'", card.model_type),
            format!("'{}'", card.interface_type),
            format!("'{}'", card.task_type),
            format!("'{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("'{}'", card.runcard_uid),
            format!("'{}'", card.pipelinecard_uid),
            format!("'{}'", card.auditcard_uid),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        format!(
            "INSERT INTO opsml_model_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }

    pub fn get_runcard_insert_query(card: &RunCardRecord) -> String {
        let mut columns = vec![
            "uid",
            "date",
            "timestamp",
            "app_env",
            "name",
            "repository",
            "major",
            "minor",
            "patch",
            "contact",
            "project",
            "tags",
            "datacard_uids",
            "modelcard_uids",
            "pipelinecard_uid",
            "artifact_uris",
            "compute_environment",
        ];
        let mut values = vec![
            format!("'{}'", card.uid),
            format!("'{}'", card.date),
            format!("{}", card.timestamp),
            format!("'{}'", card.app_env),
            format!("'{}'", card.name),
            format!("'{}'", card.repository),
            format!("{}", card.major),
            format!("{}", card.minor),
            format!("{}", card.patch),
            format!("'{}'", card.contact),
            format!("'{}'", card.project),
            format!("'{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("'{}'", serde_json::to_string(&card.datacard_uids).unwrap()),
            format!("'{}'", serde_json::to_string(&card.modelcard_uids).unwrap()),
            format!("'{}'", card.pipelinecard_uid),
            format!("'{}'", serde_json::to_string(&card.artifact_uris).unwrap()),
            format!(
                "'{}'",
                serde_json::to_string(&card.compute_environment).unwrap()
            ),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        format!(
            "INSERT INTO opsml_run_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }

    pub fn get_auditcard_insert_query(card: &AuditCardRecord) -> String {
        let mut columns = vec![
            "uid",
            "date",
            "timestamp",
            "app_env",
            "name",
            "repository",
            "major",
            "minor",
            "patch",
            "contact",
            "tags",
            "approved",
            "datacard_uids",
            "modelcard_uids",
            "runcard_uids",
        ];
        let mut values = vec![
            format!("'{}'", card.uid),
            format!("'{}'", card.date),
            format!("{}", card.timestamp),
            format!("'{}'", card.app_env),
            format!("'{}'", card.name),
            format!("'{}'", card.repository),
            format!("{}", card.major),
            format!("{}", card.minor),
            format!("{}", card.patch),
            format!("'{}'", card.contact),
            format!("'{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("{}", card.approved),
            format!("'{}'", serde_json::to_string(&card.datacard_uids).unwrap()),
            format!("'{}'", serde_json::to_string(&card.modelcard_uids).unwrap()),
            format!("'{}'", serde_json::to_string(&card.runcard_uids).unwrap()),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        format!(
            "INSERT INTO opsml_audit_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }

    pub fn get_pipelinecard_insert_query(card: &PipelineCardRecord) -> String {
        let mut columns = vec![
            "uid",
            "date",
            "timestamp",
            "app_env",
            "name",
            "repository",
            "major",
            "minor",
            "patch",
            "contact",
            "tags",
            "pipeline_code_uri",
            "datacard_uids",
            "modelcard_uids",
            "runcard_uids",
        ];
        let mut values = vec![
            format!("'{}'", card.uid),
            format!("'{}'", card.date),
            format!("{}", card.timestamp),
            format!("'{}'", card.app_env),
            format!("'{}'", card.name),
            format!("'{}'", card.repository),
            format!("{}", card.major),
            format!("{}", card.minor),
            format!("{}", card.patch),
            format!("'{}'", card.contact),
            format!("'{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("'{}'", card.pipeline_code_uri),
            format!("'{}'", serde_json::to_string(&card.datacard_uids).unwrap()),
            format!("'{}'", serde_json::to_string(&card.modelcard_uids).unwrap()),
            format!("'{}'", serde_json::to_string(&card.runcard_uids).unwrap()),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        format!(
            "INSERT INTO opsml_pipeline_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }

    pub fn get_datacard_update_query(card: &DataCardRecord) -> String {
        let mut updates = vec![
            format!("date = '{}'", card.date),
            format!("timestamp = {}", card.timestamp),
            format!("app_env = '{}'", card.app_env),
            format!("name = '{}'", card.name),
            format!("repository = '{}'", card.repository),
            format!("major = {}", card.major),
            format!("minor = {}", card.minor),
            format!("patch = {}", card.patch),
            format!("contact = '{}'", card.contact),
            format!("data_type = '{}'", card.data_type),
            format!("interface_type = '{}'", card.interface_type),
            format!("tags = '{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("runcard_uid = '{}'", card.runcard_uid),
            format!("pipelinecard_uid = '{}'", card.pipelinecard_uid),
            format!("auditcard_uid = '{}'", card.auditcard_uid),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            updates.push(format!("pre_tag = '{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            updates.push(format!("build_tag = '{}'", build_tag));
        }

        format!(
            "UPDATE opsml_data_registry SET {} WHERE uid = '{}'",
            updates.join(", "),
            card.uid
        )
    }

    pub fn get_modelcard_update_query(card: &ModelCardRecord) -> String {
        let mut updates = vec![
            format!("date = '{}'", card.date),
            format!("timestamp = {}", card.timestamp),
            format!("app_env = '{}'", card.app_env),
            format!("name = '{}'", card.name),
            format!("repository = '{}'", card.repository),
            format!("major = {}", card.major),
            format!("minor = {}", card.minor),
            format!("patch = {}", card.patch),
            format!("contact = '{}'", card.contact),
            format!("datacard_uid = '{}'", card.datacard_uid),
            format!("sample_data_type = '{}'", card.sample_data_type),
            format!("model_type = '{}'", card.model_type),
            format!("interface_type = '{}'", card.interface_type),
            format!("task_type = '{}'", card.task_type),
            format!("tags = '{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("runcard_uid = '{}'", card.runcard_uid),
            format!("pipelinecard_uid = '{}'", card.pipelinecard_uid),
            format!("auditcard_uid = '{}'", card.auditcard_uid),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            updates.push(format!("pre_tag = '{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            updates.push(format!("build_tag = '{}'", build_tag));
        }

        format!(
            "UPDATE opsml_model_registry SET {} WHERE uid = '{}'",
            updates.join(", "),
            card.uid
        )
    }

    pub fn get_runcard_update_query(card: &RunCardRecord) -> String {
        let mut updates = vec![
            format!("date = '{}'", card.date),
            format!("timestamp = {}", card.timestamp),
            format!("app_env = '{}'", card.app_env),
            format!("name = '{}'", card.name),
            format!("repository = '{}'", card.repository),
            format!("major = {}", card.major),
            format!("minor = {}", card.minor),
            format!("patch = {}", card.patch),
            format!("contact = '{}'", card.contact),
            format!("project = '{}'", card.project),
            format!("tags = '{}'", serde_json::to_string(&card.tags).unwrap()),
            format!(
                "datacard_uids = '{}'",
                serde_json::to_string(&card.datacard_uids).unwrap()
            ),
            format!(
                "modelcard_uids = '{}'",
                serde_json::to_string(&card.modelcard_uids).unwrap()
            ),
            format!("pipelinecard_uid = '{}'", card.pipelinecard_uid),
            format!(
                "artifact_uris = '{}'",
                serde_json::to_string(&card.artifact_uris).unwrap()
            ),
            format!(
                "compute_environment = '{}'",
                serde_json::to_string(&card.compute_environment).unwrap()
            ),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            updates.push(format!("pre_tag = '{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            updates.push(format!("build_tag = '{}'", build_tag));
        }

        format!(
            "UPDATE opsml_run_registry SET {} WHERE uid = '{}'",
            updates.join(", "),
            card.uid
        )
    }

    pub fn get_auditcard_update_query(card: &AuditCardRecord) -> String {
        let mut updates = vec![
            format!("date = '{}'", card.date),
            format!("timestamp = {}", card.timestamp),
            format!("app_env = '{}'", card.app_env),
            format!("name = '{}'", card.name),
            format!("repository = '{}'", card.repository),
            format!("major = {}", card.major),
            format!("minor = {}", card.minor),
            format!("patch = {}", card.patch),
            format!("contact = '{}'", card.contact),
            format!("tags = '{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("approved = {}", card.approved),
            format!(
                "datacard_uids = '{}'",
                serde_json::to_string(&card.datacard_uids).unwrap()
            ),
            format!(
                "modelcard_uids = '{}'",
                serde_json::to_string(&card.modelcard_uids).unwrap()
            ),
            format!(
                "runcard_uids = '{}'",
                serde_json::to_string(&card.runcard_uids).unwrap()
            ),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            updates.push(format!("pre_tag = '{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            updates.push(format!("build_tag = '{}'", build_tag));
        }

        format!(
            "UPDATE opsml_audit_registry SET {} WHERE uid = '{}'",
            updates.join(", "),
            card.uid
        )
    }

    pub fn get_pipelinecard_update_query(card: &PipelineCardRecord) -> String {
        let mut updates = vec![
            format!("date = '{}'", card.date),
            format!("timestamp = {}", card.timestamp),
            format!("app_env = '{}'", card.app_env),
            format!("name = '{}'", card.name),
            format!("repository = '{}'", card.repository),
            format!("major = {}", card.major),
            format!("minor = {}", card.minor),
            format!("patch = {}", card.patch),
            format!("contact = '{}'", card.contact),
            format!("tags = '{}'", serde_json::to_string(&card.tags).unwrap()),
            format!("pipeline_code_uri = '{}'", card.pipeline_code_uri),
            format!(
                "datacard_uids = '{}'",
                serde_json::to_string(&card.datacard_uids).unwrap()
            ),
            format!(
                "modelcard_uids = '{}'",
                serde_json::to_string(&card.modelcard_uids).unwrap()
            ),
            format!(
                "runcard_uids = '{}'",
                serde_json::to_string(&card.runcard_uids).unwrap()
            ),
        ];

        if let Some(pre_tag) = &card.pre_tag {
            updates.push(format!("pre_tag = '{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            updates.push(format!("build_tag = '{}'", build_tag));
        }

        format!(
            "UPDATE opsml_pipeline_registry SET {} WHERE uid = '{}'",
            updates.join(", "),
            card.uid
        )
    }
}
