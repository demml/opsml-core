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
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        if let Some(tags) = &card.tags {
            columns.push("tags");
            values.push(format!("'{}'", serde_json::to_string(tags).unwrap()));
        }

        if let Some(runcard_uid) = &card.runcard_uid {
            columns.push("runcard_uid");
            values.push(format!("'{}'", runcard_uid));
        }

        if let Some(pipelinecard_uid) = &card.pipelinecard_uid {
            columns.push("pipelinecard_uid");
            values.push(format!("'{}'", pipelinecard_uid));
        }

        if let Some(auditcard_uid) = &card.auditcard_uid {
            columns.push("auditcard_uid");
            values.push(format!("'{}'", auditcard_uid));
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
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        if let Some(tags) = &card.tags {
            columns.push("tags");
            values.push(format!("'{}'", serde_json::to_string(tags).unwrap()));
        }

        if let Some(runcard_uid) = &card.runcard_uid {
            columns.push("runcard_uid");
            values.push(format!("'{}'", runcard_uid));
        }

        if let Some(pipelinecard_uid) = &card.pipelinecard_uid {
            columns.push("pipelinecard_uid");
            values.push(format!("'{}'", pipelinecard_uid));
        }

        if let Some(auditcard_uid) = &card.auditcard_uid {
            columns.push("auditcard_uid");
            values.push(format!("'{}'", auditcard_uid));
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
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        if let Some(tags) = &card.tags {
            columns.push("tags");
            values.push(format!("'{}'", serde_json::to_string(tags).unwrap()));
        }

        if let Some(datacard_uids) = &card.datacard_uids {
            columns.push("datacard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(datacard_uids).unwrap()
            ));
        }

        if let Some(modelcard_uids) = &card.modelcard_uids {
            columns.push("modelcard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(modelcard_uids).unwrap()
            ));
        }

        if let Some(pipelinecard_uid) = &card.pipelinecard_uid {
            columns.push("pipelinecard_uid");
            values.push(format!("'{}'", pipelinecard_uid));
        }

        if let Some(artifact_uris) = &card.artifact_uris {
            columns.push("artifact_uris");
            values.push(format!(
                "'{}'",
                serde_json::to_string(artifact_uris).unwrap()
            ));
        }

        if let Some(compute_environment) = &card.compute_environment {
            columns.push("compute_environment");
            values.push(format!(
                "'{}'",
                serde_json::to_string(compute_environment).unwrap()
            ));
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
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        if let Some(tags) = &card.tags {
            columns.push("tags");
            values.push(format!("'{}'", serde_json::to_string(tags).unwrap()));
        }

        if let Some(approved) = card.approved {
            columns.push("approved");
            values.push(format!("{}", approved));
        }

        if let Some(datacard_uids) = &card.datacard_uids {
            columns.push("datacard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(datacard_uids).unwrap()
            ));
        }

        if let Some(modelcard_uids) = &card.modelcard_uids {
            columns.push("modelcard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(modelcard_uids).unwrap()
            ));
        }

        if let Some(runcard_uids) = &card.runcard_uids {
            columns.push("runcard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(runcard_uids).unwrap()
            ));
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
        ];

        if let Some(pre_tag) = &card.pre_tag {
            columns.push("pre_tag");
            values.push(format!("'{}'", pre_tag));
        }

        if let Some(build_tag) = &card.build_tag {
            columns.push("build_tag");
            values.push(format!("'{}'", build_tag));
        }

        if let Some(tags) = &card.tags {
            columns.push("tags");
            values.push(format!("'{}'", serde_json::to_string(tags).unwrap()));
        }

        if let Some(pipeline_code_uri) = &card.pipeline_code_uri {
            columns.push("pipeline_code_uri");
            values.push(format!("'{}'", pipeline_code_uri));
        }

        if let Some(datacard_uids) = &card.datacard_uids {
            columns.push("datacard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(datacard_uids).unwrap()
            ));
        }

        if let Some(modelcard_uids) = &card.modelcard_uids {
            columns.push("modelcard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(modelcard_uids).unwrap()
            ));
        }

        if let Some(runcard_uids) = &card.runcard_uids {
            columns.push("runcard_uids");
            values.push(format!(
                "'{}'",
                serde_json::to_string(runcard_uids).unwrap()
            ));
        }

        format!(
            "INSERT INTO opsml_pipeline_registry ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        )
    }
}
