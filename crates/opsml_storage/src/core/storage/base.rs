// create pyo3 async iterator
use crate::core::utils::error::StorageError;
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;
// take a stream of bytes

// create a method for Path that returns a relative path

pub trait PathExt {
    fn relative_path(&self, base: &Path) -> Result<PathBuf, StorageError>;
    fn strip_path(&self, prefix: &str) -> PathBuf;
}

impl PathExt for Path {
    fn relative_path(&self, base: &Path) -> Result<PathBuf, StorageError> {
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

// Define the StorageClient trait with common methods
pub trait StorageClient {
    fn bucket(&self) -> &str;
    fn new(bucket: String) -> Self;
    fn find(&self, path: &str) -> Result<Vec<String>, StorageError>;
    fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError>;
    fn get_object(&self, local_path: &str, remote_path: &str) -> Result<(), StorageError>;
    fn upload_file_in_chunks(
        &self,
        local_path: &Path,
        remote_path: &Path,
        chunk_size: Option<u64>,
    ) -> Result<(), StorageError>;
    fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError>;
    fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError>;
    fn delete_objects(&self, path: &str) -> Result<bool, StorageError>;
    fn delete_object(&self, path: &str) -> Result<bool, StorageError>;
    fn generate_presigned_url(&self, path: &str, expiration: u64) -> Result<String, StorageError>;
}

pub trait FileSystem<T: StorageClient> {
    fn client(&self) -> &T;
    fn new(bucket: String) -> Self;

    fn find(&self, path: &Path) -> Result<Vec<String>, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket());
        self.client().find(stripped_path.to_str().unwrap())
    }

    fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket());
        self.client().find_info(stripped_path.to_str().unwrap())
    }

    fn get(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
        // strip the paths
        let stripped_rpath = rpath.strip_path(self.client().bucket());
        let stripped_lpath = lpath.strip_path(self.client().bucket());

        if recursive {
            let stripped_lpath_clone = stripped_lpath.clone();

            // list all objects in the path
            let objects = self.client().find(stripped_rpath.to_str().unwrap())?;

            // iterate over each object and get it
            for obj in objects {
                let file_path = Path::new(obj.as_str());
                let stripped_path = file_path.strip_path(self.client().bucket());
                let relative_path = file_path.relative_path(&stripped_rpath)?;
                let local_path = stripped_lpath_clone.join(relative_path);

                self.client().get_object(
                    local_path.to_str().unwrap(),
                    stripped_path.to_str().unwrap(),
                )?;
            }
        } else {
            self.client().get_object(
                stripped_lpath.to_str().unwrap(),
                stripped_rpath.to_str().unwrap(),
            )?;
        }

        Ok(())
    }
    fn put(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_lpath = lpath.strip_path(self.client().bucket());
        let stripped_rpath = rpath.strip_path(self.client().bucket());

        if recursive {
            if !stripped_lpath.is_dir() {
                return Err(StorageError::Error(
                    "Local path must be a directory for recursive put".to_string(),
                ));
            }

            let files: Vec<PathBuf> = get_files(&stripped_lpath)?;

            for file in files {
                let stripped_lpath_clone = stripped_lpath.clone();
                let stripped_rpath_clone = stripped_rpath.clone();
                let stripped_file_path = file.strip_path(self.client().bucket());

                let relative_path = file.relative_path(&stripped_lpath_clone)?;
                let remote_path = stripped_rpath_clone.join(relative_path);

                self.client()
                    .upload_file_in_chunks(&stripped_file_path, &remote_path, None)?;
            }

            Ok(())
        } else {
            self.client()
                .upload_file_in_chunks(&stripped_lpath, &stripped_rpath, None)?;
            Ok(())
        }
    }
    fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_src = src.strip_path(self.client().bucket());
        let stripped_dest = dest.strip_path(self.client().bucket());

        if recursive {
            self.client().copy_objects(
                stripped_src.to_str().unwrap(),
                stripped_dest.to_str().unwrap(),
            )?;
        } else {
            self.client().copy_object(
                stripped_src.to_str().unwrap(),
                stripped_dest.to_str().unwrap(),
            )?;
        }

        Ok(())
    }
    fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_path = path.strip_path(self.client().bucket());

        if recursive {
            self.client()
                .delete_objects(stripped_path.to_str().unwrap())?;
        } else {
            self.client()
                .delete_object(stripped_path.to_str().unwrap())?;
        }

        Ok(())
    }
    fn exists(&self, path: &Path) -> Result<bool, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket());
        let objects = self.client().find(stripped_path.to_str().unwrap())?;

        Ok(!objects.is_empty())
    }

    fn generate_presigned_url(&self, path: &Path, expiration: u64) -> Result<String, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket());
        self.client()
            .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
    }
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
