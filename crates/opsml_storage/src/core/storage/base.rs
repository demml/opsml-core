// create pyo3 async iterator
use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use tokio::runtime::Runtime;
// take a stream of bytes
#[pyclass]
pub struct ByteIterator {
    // stream of bytes
    pub stream: Box<
        dyn futures::stream::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin + Send,
    >,
}

#[pymethods]
impl ByteIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    fn __next__<'a>(&mut self, py: Python<'a>) -> PyResult<pyo3::Bound<'a, PyBytes>> {
        let rt = Runtime::new().unwrap();

        let result = rt.block_on(async {
            let chunk = self.stream.next().await;
            println!("{:?}", chunk);
            chunk
        });

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
