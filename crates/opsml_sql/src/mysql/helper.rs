pub struct MySQLQueryHelper;

impl MySQLQueryHelper {
    pub fn get_hardware_metic_insert_query() -> String {
        "INSERT INTO opsml_hardware_metrics (run_uid, created_at, metrics) VALUES (?, ?, ?)"
            .to_string()
    }
}
