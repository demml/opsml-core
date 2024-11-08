mod py_opsml_core;
use opsml_core::core::storage::base::ClientType;
use py_opsml_core::_opsml_core::OpsmlStorageClient;
use pyo3::prelude::*;

#[pymodule]
fn _opsml_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<OpsmlStorageClient>()?;
    m.add_class::<ClientType>()?;
    Ok(())
}
