pub struct PostgresQueryHelper;

impl PostgresQueryHelper {
    pub fn get_hardware_metic_insert_query() -> String {
        "INSERT INTO opsml_hardware_metrics (
        run_uid, 
        created_at, 
        metrics
        ) 
        VALUES ($1, $2, $3)"
            .to_string()
    }
    pub fn get_projectcard_insert_query() -> String {
        "INSERT INTO opsml_project_registry (
        date, 
        uid, 
        name, 
        repository, 
        project_id, 
        major, 
        minor, 
        patch, 
        version, 
        timestamp, 
        pre_tag,
        build_tag) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
            .to_string()
    }

    pub fn get_datacard_insert_query() -> String {
        "INSERT INTO opsml_data_registry (
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
        data_type, 
        interface_type, 
        tags, 
        runcard_uid, 
        pipelinecard_uid, 
        auditcard_uid, 
        pre_tag, 
        build_tag
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)"
        .to_string()
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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)".to_string()
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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"
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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"
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
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"
            .to_string()
    }
}
