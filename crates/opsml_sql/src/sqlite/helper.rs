use crate::schemas::schema::{
    AuditCardRecord, DataCardRecord, ModelCardRecord, PipelineCardRecord, RunCardRecord,
};
use serde_json;
pub struct SqliteHelper;

impl SqliteHelper {
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
}
