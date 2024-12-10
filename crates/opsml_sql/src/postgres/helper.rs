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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)".to_string()
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

    pub fn get_datacard_update_query() -> String {
        "UPDATE opsml_data_registry SET 
        date = $1, 
        timestamp = $2, 
        app_env = $3, 
        name = $4, 
        repository = $5, 
        major = $6, 
        minor = $7, 
        patch = $8, 
        version = $9, 
        contact = $10, 
        data_type = $11, 
        interface_type = $12, 
        tags = $13, 
        runcard_uid = $14, 
        pipelinecard_uid = $15, 
        auditcard_uid = $16, 
        pre_tag = $17, 
        build_tag = $18 
        WHERE uid = $19"
            .to_string()
    }

    pub fn get_modelcard_update_query() -> String {
        "UPDATE opsml_model_registry SET 
        date = $1, 
        timestamp = $2, 
        app_env = $3, 
        name = $4, 
        repository = $5, 
        major = $6, 
        minor = $7, 
        patch = $8, 
        version = $9, 
        contact = $10, 
        datacard_uid = $11, 
        sample_data_type = $12, 
        model_type = $13, 
        interface_type = $14, 
        task_type = $15, 
        tags = $16, 
        runcard_uid = $17, 
        pipelinecard_uid = $18, 
        auditcard_uid = $19, 
        pre_tag = $20, 
        build_tag = $21 
        WHERE uid = $22"
            .to_string()
    }

    pub fn get_runcard_update_query() -> String {
        "UPDATE opsml_run_registry SET 
        date = $1, 
        timestamp = $2, 
        app_env = $3, 
        name = $4, 
        repository = $5, 
        major = $6, 
        minor = $7, 
        patch = $8, 
        version = $9, 
        contact = $10, 
        project = $11, 
        tags = $12, 
        datacard_uids = $13, 
        modelcard_uids = $14, 
        pipelinecard_uid = $15, 
        artifact_uris = $16, 
        compute_environment = $17, 
        pre_tag = $18, 
        build_tag = $19 
        WHERE uid = $20"
            .to_string()
    }

    pub fn get_auditcard_update_query() -> String {
        "UPDATE opsml_audit_registry SET 
        date = $1, 
        timestamp = $2, 
        app_env = $3, 
        name = $4, 
        repository = $5, 
        major = $6, 
        minor = $7, 
        patch = $8, 
        version = $9, 
        contact = $10, 
        tags = $11, 
        approved = $12, 
        datacard_uids = $13, 
        modelcard_uids = $14, 
        runcard_uids = $15, 
        pre_tag = $16, 
        build_tag = $17 
        WHERE uid = $18"
            .to_string()
    }

    pub fn get_pipelinecard_update_query() -> String {
        "UPDATE opsml_pipeline_registry SET 
        date = $1, 
        timestamp = $2, 
        app_env = $3, 
        name = $4, 
        repository = $5, 
        major = $6, 
        minor = $7, 
        patch = $8, 
        version = $9, 
        contact = $10, 
        tags = $11, 
        pipeline_code_uri = $12, 
        datacard_uids = $13, 
        modelcard_uids = $14, 
        runcard_uids = $15, 
        pre_tag = $16, 
        build_tag = $17 
        WHERE uid = $18"
            .to_string()
    }
}
