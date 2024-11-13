use opsml_storage::core::storage::aws::aws_storage::PyS3FSStorageClient;
use opsml_storage::core::storage::base::FileInfo;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_storage_s3(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    _m.add_class::<PyS3FSStorageClient>()?;
    _m.add_class::<FileInfo>()?;

    Ok(())
}
