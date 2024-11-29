use opsml_contracts::FileInfo;
use opsml_settings::config::{ApiSettings, OpsmlConfig, OpsmlStorageSettings, StorageType};
use opsml_storage::storage::enums::client::{get_opsml_storage_system, PyStorageClient};
use opsml_storage::storage::filesystem::PyFileSystemStorage;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pymodule]
fn _opsml_core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyFileSystemStorage>()?;
    _m.add_class::<FileInfo>()?;
    _m.add_class::<OpsmlStorageSettings>()?;
    _m.add_class::<StorageType>()?;
    _m.add_class::<OpsmlConfig>()?;
    _m.add_class::<PyStorageClient>()?;
    _m.add_class::<ApiSettings>()?;
    _m.add_function(wrap_pyfunction!(get_opsml_storage_system, _m)?)?;
    Ok(())
}
