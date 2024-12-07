-- Add migration script here
-- MySQL Migration Script

-- DataSchema
CREATE TABLE IF NOT EXISTS opsml_data_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSON,
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
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSON,
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
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSON,
    datacard_uids JSON,
    modelcard_uids JSON,
    pipelinecard_uid VARCHAR(64),
    project VARCHAR(64),
    artifact_uris JSON,
    compute_environment JSON
);

-- AuditSchema
CREATE TABLE IF NOT EXISTS opsml_audit_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSON,
    approved BOOLEAN,
    datacard_uids JSON,
    modelcard_uids JSON,
    runcard_uids JSON
);

-- PipelineSchema
CREATE TABLE IF NOT EXISTS opsml_pipeline_registry (
    uid VARCHAR(64) PRIMARY KEY,
    date VARCHAR(32),
    timestamp BIGINT,
    app_env VARCHAR(32) DEFAULT 'development',
    name VARCHAR(128),
    repository VARCHAR(128),
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(16),
    build_tag VARCHAR(16),
    version VARCHAR(64),
    contact VARCHAR(64),
    tags JSON,
    pipeline_code_uri VARCHAR(256),
    datacard_uids JSON,
    modelcard_uids JSON,
    runcard_uids JSON
);

-- ProjectSchema
CREATE TABLE IF NOT EXISTS opsml_project_registry (
    uid VARCHAR(64),
    name VARCHAR(128),
    repository VARCHAR(128),
    project_id INT PRIMARY KEY AUTO_INCREMENT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
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
    idx INT PRIMARY KEY AUTO_INCREMENT
);

-- ParameterSchema
CREATE TABLE IF NOT EXISTS opsml_run_parameters (
    run_uid VARCHAR(64),
    name VARCHAR(128),
    value VARCHAR(128),
    date_ts VARCHAR(64) DEFAULT (CURRENT_TIMESTAMP),
    idx INT PRIMARY KEY AUTO_INCREMENT
);

-- HardwareMetricSchema
CREATE TABLE IF NOT EXISTS opsml_run_hardware_metrics (
    run_uid VARCHAR(64) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metrics JSON,
    idx INT PRIMARY KEY AUTO_INCREMENT
);