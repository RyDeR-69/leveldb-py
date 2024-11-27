#![allow(clippy::too_many_arguments)]
use ::leveldb::database::cache::Cache;
use ::leveldb::iterator::Iterable;
use ::leveldb::{database, options};
use leveldb_sys::Compression;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyTypeInfo;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[pyclass]
pub struct BytesKey(Vec<u8>);

impl db_key::Key for BytesKey {
    fn from_u8(key: &[u8]) -> Self {
        BytesKey(key.to_vec())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0)
    }
}

#[derive(Clone, Debug)]
#[pyclass(name = "OpenOptions")]
struct PyOpenOptions {
    /// create the database if missing
    ///
    /// default: false
    pub create_if_missing: bool,
    /// report an error if the DB already exists instead of opening.
    ///
    /// default: false
    pub error_if_exists: bool,
    /// paranoid checks make the database report an error as soon as
    /// corruption is detected.
    ///
    /// default: false
    pub paranoid_checks: bool,
    /// Override the size of the write buffer to use.
    ///
    /// default: None
    pub write_buffer_size: Option<usize>,
    /// Override the max number of open files.
    ///
    /// default: None
    pub max_open_files: Option<i32>,
    /// Override the size of the blocks leveldb uses for writing and caching.
    ///
    /// default: None
    pub block_size: Option<usize>,
    /// Override the interval between restart points.
    ///
    /// default: None
    pub block_restart_interval: Option<i32>,
    /// Define whether leveldb should write compressed or not.
    ///
    /// default: Compression::No
    pub compression: bool,
    /// A cache to use during read operations.
    ///
    /// default: None
    pub cache: Option<usize>,
}

#[pymethods]
impl PyOpenOptions {
    #[pyo3(
        signature = (
            create_if_missing = None,
            error_if_exists = None,
            paranoid_checks = None,
            write_buffer_size = None,
            max_open_files = None,
            block_size = None,
            block_restart_interval = None,
            compression = None,
            cache = None
        )
    )]
    #[new]
    fn new(
        create_if_missing: Option<bool>,
        error_if_exists: Option<bool>,
        paranoid_checks: Option<bool>,
        write_buffer_size: Option<usize>,
        max_open_files: Option<i32>,
        block_size: Option<usize>,
        block_restart_interval: Option<i32>,
        compression: Option<bool>,
        cache: Option<usize>,
    ) -> Self {
        PyOpenOptions {
            create_if_missing: create_if_missing.unwrap_or(false),
            error_if_exists: error_if_exists.unwrap_or(false),
            paranoid_checks: paranoid_checks.unwrap_or(false),
            write_buffer_size,
            max_open_files,
            block_size,
            block_restart_interval,
            compression: compression.unwrap_or(false),
            cache,
        }
    }
}

impl From<PyOpenOptions> for options::Options {
    fn from(py_options: PyOpenOptions) -> Self {
        let mut options = options::Options::new();
        options.create_if_missing = py_options.create_if_missing;
        options.error_if_exists = py_options.error_if_exists;
        options.paranoid_checks = py_options.paranoid_checks;
        options.write_buffer_size = py_options.write_buffer_size;
        options.max_open_files = py_options.max_open_files;
        options.block_size = py_options.block_size;
        options.block_restart_interval = py_options.block_restart_interval;
        options.compression = if py_options.compression {
            Compression::Snappy
        } else {
            Compression::No
        };
        options.cache = py_options.cache.map(Cache::new);
        options
    }
}

#[derive(Clone, Debug)]
#[pyclass(name = "ReadOptions")]
struct PyReadOptions {
    /// Whether to verify the saved checksums on read.
    ///
    /// default: false
    pub verify_checksums: bool,
    /// Whether to fill the internal cache with the
    /// results of the read.
    ///
    /// default: true
    pub fill_cache: bool,
    // snapshot unimplemented
}

#[pymethods]
impl PyReadOptions {
    #[pyo3(signature = (verify_checksums = None, fill_cache = None))]
    #[new]
    fn new(verify_checksums: Option<bool>, fill_cache: Option<bool>) -> Self {
        PyReadOptions {
            verify_checksums: verify_checksums.unwrap_or(false),
            fill_cache: fill_cache.unwrap_or(true),
        }
    }
}

impl From<PyReadOptions> for options::ReadOptions<'_, BytesKey> {
    fn from(py_options: PyReadOptions) -> Self {
        let mut options = options::ReadOptions::new();
        options.verify_checksums = py_options.verify_checksums;
        options.fill_cache = py_options.fill_cache;
        options
    }
}

#[derive(Clone, Debug)]
#[pyclass(name = "WriteOptions")]
struct PyWriteOptions {
    /// fsync before acknowledging a write operation.
    ///
    /// default: false
    pub sync: bool,
}

#[pymethods]
impl PyWriteOptions {
    #[pyo3(signature = (sync = None))]
    #[new]
    fn new(sync: Option<bool>) -> Self {
        PyWriteOptions {
            sync: sync.unwrap_or(false),
        }
    }
}

impl From<PyWriteOptions> for options::WriteOptions {
    fn from(py_options: PyWriteOptions) -> Self {
        let mut options = options::WriteOptions::new();
        options.sync = py_options.sync;
        options
    }
}

#[derive(FromPyObject)]
enum PyOpenOptionsTypes {
    Typed(PyOpenOptions),
    #[allow(dead_code)]
    Raw(HashMap<String, String>),
}

#[derive(FromPyObject)]
enum PyReadOptionsTypes {
    Typed(PyReadOptions),
    #[allow(dead_code)]
    Raw(HashMap<String, String>),
}

#[allow(dead_code)]
#[derive(FromPyObject)]
enum PyWriteOptionsTypes {
    Typed(PyWriteOptions),
    Raw(HashMap<String, String>),
}

#[pyclass(unsendable)]
struct PyIterator {
    // db: Arc<database::Database<BytesKey>>,
    iter: RefCell<database::iterator::Iterator<'static, BytesKey>>,
}

#[pymethods]
impl PyIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__<'a>(
        slf: PyRef<Self>,
        py: Python<'a>,
    ) -> Option<(Bound<'a, PyBytes>, Bound<'a, PyBytes>)> {
        let mut iter = slf.iter.borrow_mut();
        if let Some((key, value)) = iter.next() {
            // Convert Rust Vec<u8> to Python bytes
            let py_key = PyBytes::new(py, &key.0);
            let py_value = PyBytes::new(py, &value);
            Some((py_key, py_value))
        } else {
            None
        }
    }
}

#[pyclass(name = "Database")]
struct PyDatabase(Arc<database::Database<BytesKey>>);

#[pymethods]
impl PyDatabase {
    #[new]
    fn new(path: &str, open_options: PyOpenOptionsTypes) -> PyResult<Self> {
        let path = std::path::Path::new(path);
        let options = match open_options {
            PyOpenOptionsTypes::Typed(options) => options.into(),
            PyOpenOptionsTypes::Raw(_) => {
                unimplemented!("Raw options are not implemented yet");
            }
        };
        let db = database::Database::open(path, options).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "Unable to open database: {}",
                e
            ))
        })?;
        Ok(PyDatabase(Arc::new(db)))
    }

    fn iter(&self, read_options: PyReadOptionsTypes) -> PyResult<PyIterator> {
        let options = match read_options {
            PyReadOptionsTypes::Typed(options) => options.into(),
            PyReadOptionsTypes::Raw(_) => {
                unimplemented!("Raw options are not implemented yet");
            }
        };
        let iter = self.0.iter(options);

        // SAFETY: We are extending the lifetime of the iterator to 'static.
        // This is safe because the Arc ensures that the Database lives as long as the iterator.
        let iter_static: database::iterator::Iterator<'static, BytesKey> =
            unsafe { std::mem::transmute(iter) };

        Ok(PyIterator {
            // db: self.0.clone(),
            iter: RefCell::new(iter_static),
        })
    }
}

/// Define the Python module
#[pymodule]
fn leveldb(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDatabase>()?;
    m.add_class::<PyOpenOptions>()?;
    m.add_class::<PyReadOptions>()?;
    m.add_class::<PyWriteOptions>()?;
    m.add_class::<PyIterator>()?;

    let typing = py.import("typing")?;
    let union = typing.getattr("Union")?;
    m.add(
        "PyOpenOptionsTypes",
        union.get_item(PyOpenOptions::type_object(py))?,
    )?;
    m.add(
        "PyReadOptionsTypes",
        union.get_item(PyReadOptions::type_object(py))?,
    )?;
    m.add(
        "PyWriteOptionsTypes",
        union.get_item(PyWriteOptions::type_object(py))?,
    )?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
