use opsml_storage::core::storage::google::google_storage::GCSFSStorageClient;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_storage(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<GCSFSStorageClient>()?;

    Ok(())
}
