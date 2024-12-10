/// this file contains helper logic for generating sql queries across different databases
use crate::base::CardSQLTableNames;
pub struct SqliteQueryHelper;

impl SqliteQueryHelper {
    pub fn get_hardware_metic_insert_query() -> String {
        format!(
            "INSERT INTO {} (run_uid, created_at, metrics) VALUES (?, ?, ?)",
            CardSQLTableNames::HardwareMetrics
        )
        .to_string()
    }

    pub fn get_projectcard_insert_query() -> String {
        format!("INSERT INTO {} (date, uid, name, repository, project_id, major, minor, patch, version, timestamp, pre_tag, build_tag) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", CardSQLTableNames::Project)
            .to_string()
    }

    pub fn get_datacard_insert_query() -> String {
        format!("INSERT INTO {} (uid, date, timestamp, app_env, name, repository, major, minor, patch, version, contact, data_type, interface_type, tags, runcard_uid, pipelinecard_uid, auditcard_uid, pre_tag, build_tag) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", CardSQLTableNames::Data)
            .to_string()
    }

    pub fn get_modelcard_insert_query() -> String {
        format!(
            "INSERT INTO {} (
        uid, 
        date, 
        timestamp, 
        app_env, 
        name, 
        repository, 
        major, 
        minor, 
        patch, 
        version, 
        contact, 
        datacard_uid, 
        sample_data_type, 
        model_type, 
        interface_type, 
        task_type, 
        tags, 
        runcard_uid, 
        pipelinecard_uid, 
        auditcard_uid, 
        pre_tag, 
        build_tag
        ) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            CardSQLTableNames::Model
        )
        .to_string()
    }

    pub fn get_runcard_insert_query() -> String {
        format!(
            "INSERT INTO {} (
        uid, 
        date, 
        timestamp, 
        app_env, 
        name, 
        repository, 
        major, 
        minor, 
        patch, 
        version, 
        contact, 
        project, 
        tags, 
        datacard_uids,
        modelcard_uids, 
        pipelinecard_uid, 
        artifact_uris, 
        compute_environment, 
        pre_tag, 
        build_tag
        ) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            CardSQLTableNames::Run
        )
        .to_string()
    }

    pub fn get_auditcard_insert_query() -> String {
        format!(
            "INSERT INTO {} (
        uid, 
        date, 
        timestamp, 
        app_env, 
        name, 
        repository, 
        major, 
        minor, 
        patch, 
        version, 
        contact, 
        tags, 
        approved, 
        datacard_uids, 
        modelcard_uids, 
        runcard_uids, 
        pre_tag, 
        build_tag
        ) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            CardSQLTableNames::Audit
        )
        .to_string()
    }

    pub fn get_pipelinecard_insert_query() -> String {
        format!(
            "INSERT INTO {} (
        uid, 
        date, 
        timestamp, 
        app_env, 
        name, 
        repository, 
        major, 
        minor, 
        patch, 
        version, 
        contact, 
        tags, 
        pipeline_code_uri, 
        datacard_uids, 
        modelcard_uids, 
        runcard_uids, 
        pre_tag, 
        build_tag
        ) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            CardSQLTableNames::Pipeline
        )
        .to_string()
    }

    pub fn get_datacard_update_query() -> String {
        format!(
            "UPDATE {} SET 
        date = ?, 
        timestamp = ?, 
        app_env = ?, 
        name = ?, 
        repository = ?, 
        major = ?, 
        minor = ?, 
        patch = ?, 
        version = ?, 
        contact = ?, 
        data_type = ?, 
        interface_type = ?, 
        tags = ?, 
        runcard_uid = ?, 
        pipelinecard_uid = ?, 
        auditcard_uid = ?, 
        pre_tag = ?, 
        build_tag = ? 
        WHERE uid = ?",
            CardSQLTableNames::Data
        )
        .to_string()
    }

    pub fn get_modelcard_update_query() -> String {
        format!(
            "UPDATE {} SET 
        date = ?, 
        timestamp = ?, 
        app_env = ?, 
        name = ?, 
        repository = ?, 
        major = ?, 
        minor = ?, 
        patch = ?, 
        version = ?, 
        contact = ?, 
        datacard_uid = ?, 
        sample_data_type = ?, 
        model_type = ?, 
        interface_type = ?, 
        task_type = ?, 
        tags = ?, 
        runcard_uid = ?, 
        pipelinecard_uid = ?, 
        auditcard_uid = ?, 
        pre_tag = ?, 
        build_tag = ? 
        WHERE uid = ?",
            CardSQLTableNames::Model
        )
        .to_string()
    }

    pub fn get_runcard_update_query() -> String {
        format!(
            "UPDATE {} SET 
        date = ?, 
        timestamp = ?, 
        app_env = ?, 
        name = ?, 
        repository = ?, 
        major = ?, 
        minor = ?, 
        patch = ?, 
        version = ?, 
        contact = ?, 
        project = ?, 
        tags = ?, 
        datacard_uids = ?, 
        modelcard_uids = ?, 
        pipelinecard_uid = ?, 
        artifact_uris = ?, 
        compute_environment = ?, 
        pre_tag = ?, 
        build_tag = ? 
        WHERE uid = ?",
            CardSQLTableNames::Run
        )
        .to_string()
    }

    pub fn get_auditcard_update_query() -> String {
        format!(
            "UPDATE {} SET 
        date = ?, 
        timestamp = ?, 
        app_env = ?, 
        name = ?, 
        repository = ?, 
        major = ?, 
        minor = ?, 
        patch = ?, 
        version = ?, 
        contact = ?, 
        tags = ?, 
        approved = ?, 
        datacard_uids = ?, 
        modelcard_uids = ?, 
        runcard_uids = ?, 
        pre_tag = ?, 
        build_tag = ?
        WHERE uid = ?",
            CardSQLTableNames::Audit
        )
        .to_string()
    }

    pub fn get_pipelinecard_update_query() -> String {
        format!(
            "UPDATE {} SET 
        date = ?, 
        timestamp = ?, 
        app_env = ?, 
        name = ?, 
        repository = ?, 
        major = ?, 
        minor = ?, 
        patch = ?, 
        version = ?, 
        contact = ?, 
        tags = ?, 
        pipeline_code_uri = ?, 
        datacard_uids = ?, 
        modelcard_uids = ?, 
        runcard_uids = ?, 
        pre_tag = ?, 
        build_tag = ? 
        WHERE uid = ?",
            CardSQLTableNames::Pipeline
        )
        .to_string()
    }
}
