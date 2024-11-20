use opsml_storage::core::storage::aws::aws_storage::PyS3FSStorageClient;
use opsml_storage::core::storage::base::{FileInfo, StorageSettings, StorageType};
use opsml_storage::core::storage::google::google_storage::PyGCSFSStorageClient;
use opsml_storage::core::storage::http::PyHttpFSStorageClient;
use opsml_storage::core::storage::local::PyLocalFSStorageClient;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_storage_local(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyS3FSStorageClient>()?;
    _m.add_class::<PyHttpFSStorageClient>()?;
    _m.add_class::<PyLocalFSStorageClient>()?;
    _m.add_class::<PyGCSFSStorageClient>()?;
    _m.add_class::<FileInfo>()?;
    _m.add_class::<StorageSettings>()?;
    _m.add_class::<StorageType>()?;

    Ok(())
}
