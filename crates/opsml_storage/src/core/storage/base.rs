// create pyo3 async iterator
use crate::core::utils::error::StorageError;
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;
// take a stream of bytes

// create a method for Path that returns a relative path

pub trait PathExt {
    fn relative_path(&self, base: &PathBuf) -> Result<PathBuf, StorageError>;
    fn strip_path(&self, prefix: &str) -> PathBuf;
}

impl PathExt for Path {
    fn relative_path(&self, base: &PathBuf) -> Result<PathBuf, StorageError> {
        self.strip_prefix(base)
            .map_err(|e| StorageError::Error(format!("Failed to get relative path: {}", e)))
            .map(|p| p.to_path_buf())
    }

    fn strip_path(&self, prefix: &str) -> PathBuf {
        self.strip_prefix(prefix).unwrap_or(self).to_path_buf()
    }
}

/// Get all files in a directory (including subdirectories)
pub fn get_files(path: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let files: Vec<_> = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

pub trait FileSystem {
    fn new(bucket: String) -> Self;
    fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError>;
    fn find(&self, path: &Path) -> Result<Vec<String>, StorageError>;
    fn get(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError>;
    fn put(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError>;
    fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError>;
    fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError>;
    fn exists(&self, path: &Path) -> Result<bool, StorageError>;
    fn generate_presigned_url(&self, path: &Path, expiration: u64) -> Result<String, StorageError>;
}

#[derive(Debug)]
#[pyclass]
pub struct FileInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub size: i64,
    #[pyo3(get)]
    pub object_type: String,
    #[pyo3(get)]
    pub created: String,
    #[pyo3(get)]
    pub suffix: String,
}
