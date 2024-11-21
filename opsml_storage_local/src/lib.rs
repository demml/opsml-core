use opsml_settings::config::{ApiSettings, OpsmlConfig, OpsmlStorageSettings, StorageType};
use opsml_storage::core::storage::aws::aws_storage::PyS3FSStorageClient;
use opsml_storage::core::storage::base::FileInfo;
use opsml_storage::core::storage::enums::{get_opsml_storage_system, PyStorageClient};
use opsml_storage::core::storage::google::google_storage::PyGCSFSStorageClient;
use opsml_storage::core::storage::http::PyHttpFSStorageClient;
use opsml_storage::core::storage::local::PyLocalFSStorageClient;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pymodule]
fn _opsml_storage_local(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyS3FSStorageClient>()?;
    _m.add_class::<PyHttpFSStorageClient>()?;
    _m.add_class::<PyLocalFSStorageClient>()?;
    _m.add_class::<PyGCSFSStorageClient>()?;
    _m.add_class::<FileInfo>()?;
    _m.add_class::<OpsmlStorageSettings>()?;
    _m.add_class::<StorageType>()?;
    _m.add_class::<OpsmlConfig>()?;
    _m.add_class::<PyStorageClient>()?;
    _m.add_class::<ApiSettings>()?;
    _m.add_function(wrap_pyfunction!(get_opsml_storage_system, _m)?)?;
    Ok(())
}
