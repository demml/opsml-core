-- DataSchema
CREATE TABLE IF NOT EXISTS opsml_data_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(64),
    build_tag VARCHAR(64),
    contact VARCHAR(64),
    tags JSONB,
    data_type VARCHAR(64),
    runcard_uid VARCHAR(64),
    pipelinecard_uid VARCHAR(64),
    auditcard_uid VARCHAR(64),
    interface_type VARCHAR(64) NOT NULL DEFAULT 'undefined'
);

-- ModelSchema
CREATE TABLE IF NOT EXISTS opsml_model_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSONB,
    datacard_uid VARCHAR(64),
    sample_data_type VARCHAR(64),
    model_type VARCHAR(64),
    runcard_uid VARCHAR(64),
    pipelinecard_uid VARCHAR(64),
    auditcard_uid VARCHAR(64),
    interface_type VARCHAR(64) NOT NULL DEFAULT 'undefined',
    task_type VARCHAR(64) NOT NULL DEFAULT 'undefined'
);

-- RunSchema
CREATE TABLE IF NOT EXISTS opsml_run_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSONB,
    datacard_uids JSONB,
    modelcard_uids JSONB,
    pipelinecard_uid VARCHAR(64),
    project VARCHAR(64),
    artifact_uris JSONB,
    compute_environment JSONB
);

-- AuditSchema
CREATE TABLE IF NOT EXISTS opsml_audit_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSONB,
    approved BOOLEAN,
    datacard_uids JSONB,
    modelcard_uids JSONB,
    runcard_uids JSONB
);

-- PipelineSchema
CREATE TABLE IF NOT EXISTS opsml_pipeline_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSONB,
    pipeline_code_uri VARCHAR(256),
    datacard_uids JSONB,
    modelcard_uids JSONB,
    runcard_uids JSONB
);

-- ProjectSchema
CREATE TABLE IF NOT EXISTS opsml_project_registry (
    uid VARCHAR(64),
    name VARCHAR(128),
    repository VARCHAR(128),
    project_id SERIAL PRIMARY KEY,
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    timestamp BIGINT
);

-- MetricSchema
CREATE TABLE IF NOT EXISTS opsml_run_metrics (
    run_uid VARCHAR(64),
    name VARCHAR(128),
    value FLOAT,
    step INT,
    timestamp BIGINT,
    date_ts VARCHAR(64) DEFAULT (CURRENT_TIMESTAMP),
    idx SERIAL PRIMARY KEY
);

-- ParameterSchema
CREATE TABLE IF NOT EXISTS opsml_run_parameters (
    run_uid VARCHAR(64),
    name VARCHAR(128),
    value VARCHAR(128),
    date_ts VARCHAR(64) DEFAULT (CURRENT_TIMESTAMP),
    idx SERIAL PRIMARY KEY
);

-- HardwareMetricSchema
CREATE TABLE IF NOT EXISTS opsml_run_hardware_metrics (
    run_uid VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    metrics JSONB,
    idx SERIAL PRIMARY KEY
);