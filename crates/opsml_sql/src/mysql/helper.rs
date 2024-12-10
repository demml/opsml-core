use crate::base::CardSQLTableNames;

pub struct MySQLQueryHelper;

impl MySQLQueryHelper {
    pub fn get_run_parameter_insert_query() -> String {
        format!(
            "INSERT INTO {} (
                run_uid, 
                name, 
                value
            ) VALUES (?, ?, ?)",
            CardSQLTableNames::Parameters
        )
        .to_string()
    }

    pub fn get_run_parameter_query(names: Option<&Vec<&str>>) -> String {
        let mut query = format!(
            "SELECT *
            FROM {}
            WHERE run_uid = ?",
            CardSQLTableNames::Parameters
        );

        // loop through names and bind them. First name = and and others are or
        if names.is_some() {
            let names = names.unwrap();
            for (idx, name) in names.iter().enumerate() {
                if idx == 0 {
                    query.push_str(format!(" AND (name = {}", name).as_str());
                } else {
                    query.push_str(format!(" OR name = {}", name).as_str());
                }
            }
            query.push_str(")");
        }

        query
    }
    pub fn get_hardware_metric_insert_query() -> String {
        format!(
            "INSERT INTO {} (
                run_uid, 
                created_at, 
                cpu_percent_utilization, 
                cpu_percent_per_core, 
                compute_overall, 
                compute_utilized, 
                load_avg, 
                sys_ram_total, 
                sys_ram_used, 
                sys_ram_available, 
                sys_ram_percent_used, 
                sys_swap_total, 
                sys_swap_used, 
                sys_swap_free, 
                sys_swap_percent, 
                bytes_recv, 
                bytes_sent, 
                gpu_percent_utilization, 
                gpu_percent_per_core
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            CardSQLTableNames::HardwareMetrics
        )
        .to_string()
    }
    pub fn get_projectcard_insert_query() -> String {
        "INSERT INTO opsml_project_registry (date, uid, name, repository, project_id, major, minor, patch, version, timestamp, pre_tag, build_tag) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()
    }

    pub fn get_datacard_insert_query() -> String {
        "INSERT INTO opsml_data_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, version, contact, data_type, interface_type, tags, runcard_uid, pipelinecard_uid, auditcard_uid, pre_tag, build_tag) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()
    }

    pub fn get_modelcard_insert_query() -> String {
        "INSERT INTO opsml_model_registry (
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            .to_string()
    }

    pub fn get_runcard_insert_query() -> String {
        "INSERT INTO opsml_run_registry (
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            .to_string()
    }

    pub fn get_auditcard_insert_query() -> String {
        "INSERT INTO opsml_audit_registry (
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            .to_string()
    }

    pub fn get_pipelinecard_insert_query() -> String {
        "INSERT INTO opsml_pipeline_registry (
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
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            .to_string()
    }

    pub fn get_datacard_update_query() -> String {
        "UPDATE opsml_data_registry SET 
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
        WHERE uid = ?"
            .to_string()
    }

    pub fn get_modelcard_update_query() -> String {
        "UPDATE opsml_model_registry SET 
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
        WHERE uid = ?"
            .to_string()
    }

    pub fn get_runcard_update_query() -> String {
        "UPDATE opsml_run_registry SET 
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
        WHERE uid = ?"
            .to_string()
    }

    pub fn get_auditcard_update_query() -> String {
        "UPDATE opsml_audit_registry SET 
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
        WHERE uid = ?"
            .to_string()
    }

    pub fn get_pipelinecard_update_query() -> String {
        "UPDATE opsml_pipeline_registry SET 
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
        WHERE uid = ?"
            .to_string()
    }
}
