// create pyo3 async iterator
use crate::core::utils::error::StorageError;
use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;
use tokio::runtime::Runtime;
// take a stream of bytes

pub trait FileSystem {
    fn find(&self, path: PathBuf) -> Result<Vec<String>, StorageError>;
    fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError>;
    fn put(&self, lpath: PathBuf, rpath: PathBuf) -> Result<(), StorageError>;
    fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> Result<(), StorageError>;
    fn rm(&self, path: PathBuf, recursive: bool) -> Result<(), StorageError>;
    fn exists(&self, path: PathBuf) -> Result<bool, StorageError>;
    fn generate_presigned_url(
        &self,
        path: PathBuf,
        expiration: u64,
    ) -> Result<String, StorageError>;
}

#[pyclass]
pub struct ByteIterator {
    // stream of bytes
    pub stream:
        Box<dyn futures::stream::Stream<Item = Result<bytes::Bytes, StorageError>> + Unpin + Send>,
    pub runtime: Runtime,
}

#[pymethods]
impl ByteIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    fn __next__<'a>(&mut self, py: Python<'a>) -> PyResult<pyo3::Bound<'a, PyBytes>> {
        let result = self.runtime.block_on(async { self.stream.next().await });

        match result {
            Some(Ok(chunk)) => Ok(PyBytes::new_bound(py, &chunk)),
            Some(Err(e)) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
            None => Err(PyErr::new::<pyo3::exceptions::PyStopIteration, _>(
                "Stream exhausted",
            )),
        }
    }
}
