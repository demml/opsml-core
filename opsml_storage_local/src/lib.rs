use opsml_storage::core::storage::base::FileInfo;
use opsml_storage::core::storage::local::PyLocalFSStorageClient;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_storage_local(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyLocalFSStorageClient>()?;
    _m.add_class::<FileInfo>()?;

    Ok(())
}
