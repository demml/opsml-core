use pyo3::prelude::*;

#[cfg(feature = "google_storage")]
use opsml_core::core::storage::google::google_storage::GCSFSStorageClient;

#[pymodule]
fn _opsml_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(feature = "google_storage")]
    m.add_class::<GCSFSStorageClient>()?;

    Ok(())
}
