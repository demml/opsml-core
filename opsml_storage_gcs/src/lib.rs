use opsml_storage::core::storage::base::FileInfo;
use opsml_storage::core::storage::google::google_storage::PyS3GCSFSStorageClient;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_storage_gcs(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyS3GCSFSStorageClient>()?;
    _m.add_class::<FileInfo>()?;

    Ok(())
}
