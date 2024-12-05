-- Add migration script here
-- SQLite Migration Script

-- DataSchema
CREATE TABLE IF NOT EXISTS opsml_data_registry (
    uid TEXT PRIMARY KEY,
    date TEXT,
    timestamp INTEGER,
    app_env TEXT DEFAULT 'development',
    name TEXT,
    repository TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    contact TEXT,
    tags TEXT,
    data_type TEXT,
    runcard_uid TEXT,
    pipelinecard_uid TEXT,
    auditcard_uid TEXT,
    interface_type TEXT NOT NULL DEFAULT 'undefined'
);

-- ModelSchema
CREATE TABLE IF NOT EXISTS opsml_model_registry (
    uid TEXT PRIMARY KEY,
    date TEXT,
    timestamp INTEGER,
    app_env TEXT DEFAULT 'development',
    name TEXT,
    repository TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    contact TEXT,
    tags TEXT,
    datacard_uid TEXT,
    sample_data_type TEXT,
    model_type TEXT,
    runcard_uid TEXT,
    pipelinecard_uid TEXT,
    auditcard_uid TEXT,
    interface_type TEXT NOT NULL DEFAULT 'undefined',
    task_type TEXT NOT NULL DEFAULT 'undefined'
);

-- RunSchema
CREATE TABLE IF NOT EXISTS opsml_run_registry (
    uid TEXT PRIMARY KEY,
    date TEXT,
    timestamp INTEGER,
    app_env TEXT DEFAULT 'development',
    name TEXT,
    repository TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    contact TEXT,
    tags TEXT,
    datacard_uids TEXT,
    modelcard_uids TEXT,
    pipelinecard_uid TEXT,
    project TEXT,
    artifact_uris TEXT,
    compute_environment TEXT
);

-- AuditSchema
CREATE TABLE IF NOT EXISTS opsml_audit_registry (
    uid TEXT PRIMARY KEY,
    date TEXT,
    timestamp INTEGER,
    app_env TEXT DEFAULT 'development',
    name TEXT,
    repository TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    contact TEXT,
    tags TEXT,
    approved BOOLEAN,
    datacard_uids TEXT,
    modelcard_uids TEXT,
    runcard_uids TEXT
);

-- PipelineSchema
CREATE TABLE IF NOT EXISTS opsml_pipeline_registry (
    uid TEXT PRIMARY KEY,
    date TEXT,
    timestamp INTEGER,
    app_env TEXT DEFAULT 'development',
    name TEXT,
    repository TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    contact TEXT,
    tags TEXT,
    pipeline_code_uri TEXT,
    datacard_uids TEXT,
    modelcard_uids TEXT,
    runcard_uids TEXT
);

-- ProjectSchema
CREATE TABLE IF NOT EXISTS opsml_project_registry (
    uid TEXT,
    name TEXT,
    repository TEXT,
    project_id INTEGER PRIMARY KEY,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL,
    pre_tag VARCHAR(255),
    build_tag VARCHAR(255),
    timestamp INTEGER
);

-- MetricSchema
CREATE TABLE IF NOT EXISTS opsml_run_metrics (
    run_uid TEXT,
    name TEXT,
    value REAL,
    step INTEGER,
    timestamp INTEGER,
    date_ts TEXT DEFAULT (datetime('now')),
    idx INTEGER PRIMARY KEY
);

-- ParameterSchema
CREATE TABLE IF NOT EXISTS opsml_run_parameters (
    run_uid TEXT,
    name TEXT,
    value TEXT,
    date_ts TEXT DEFAULT (datetime('now')),
    idx INTEGER PRIMARY KEY
);

-- HardwareMetricSchema
CREATE TABLE IF NOT EXISTS opsml_run_hardware_metrics (
    run_uid TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    metrics TEXT,
    idx INTEGER PRIMARY KEY
);