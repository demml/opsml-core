use crate::core::storage::base::{FileInfo, FileSystem, StorageClient};
use crate::core::utils::error::StorageError;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

pub struct LocalStorageClient {
    pub root: PathBuf,
}

impl StorageClient for LocalStorageClient {
    fn bucket(&self) -> &str {
        self.root.to_str().unwrap()
    }

    fn new(root: String) -> Self {
        Self {
            root: PathBuf::from(root),
        }
    }

    fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
        let src_path = self.root.join(rpath);
        let dest_path = Path::new(lpath);

        if !src_path.exists() {
            return Err(StorageError::Error(format!(
                "Source path does not exist: {}",
                src_path.display()
            )));
        }

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))?;
        }

        fs::copy(&src_path, dest_path)
            .map_err(|e| StorageError::Error(format!("Unable to copy file: {}", e)))?;

        Ok(())
    }

    fn generate_presigned_url(&self, path: &str, _expiration: u64) -> Result<String, StorageError> {
        let full_path = self.root.join(path);
        if full_path.exists() {
            Ok(full_path.to_str().unwrap().to_string())
        } else {
            Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )))
        }
    }

    fn upload_file_in_chunks(
        &self,
        lpath: &Path,
        rpath: &Path,
        _chunk_size: Option<u64>,
    ) -> Result<(), StorageError> {
        let dest_path = self.root.join(rpath);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))?;
        }

        fs::copy(lpath, &dest_path)
            .map_err(|e| StorageError::Error(format!("Unable to copy file: {}", e)))?;

        Ok(())
    }

    fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let full_path = self.root.join(path);
        if !full_path.exists() {
            return Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )));
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(full_path) {
            let entry = entry
                .map_err(|e| StorageError::Error(format!("Unable to read directory: {}", e)))?;
            if entry.file_type().is_file() {
                files.push(entry.path().to_str().unwrap().to_string());
            }
        }

        Ok(files)
    }

    fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
        let full_path = self.root.join(path);
        if !full_path.exists() {
            return Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )));
        }

        let mut files_info = Vec::new();
        for entry in WalkDir::new(full_path) {
            let entry = entry
                .map_err(|e| StorageError::Error(format!("Unable to read directory: {}", e)))?;
            if entry.file_type().is_file() {
                let metadata = entry
                    .metadata()
                    .map_err(|e| StorageError::Error(format!("Unable to read metadata: {}", e)))?;
                let created = metadata
                    .created()
                    .unwrap_or(SystemTime::now())
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                let file_info = FileInfo {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    size: metadata.len() as i64,
                    object_type: "file".to_string(),
                    created,
                    suffix: entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("")
                        .to_string(),
                };
                files_info.push(file_info);
            }
        }

        Ok(files_info)
    }

    fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
        let src_path = self.root.join(src);
        let dest_path = self.root.join(dest);

        if !src_path.exists() {
            return Err(StorageError::Error(format!(
                "Source path does not exist: {}",
                src_path.display()
            )));
        }

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))?;
        }

        fs::copy(&src_path, &dest_path)
            .map_err(|e| StorageError::Error(format!("Unable to copy file: {}", e)))?;

        Ok(true)
    }

    fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
        let src_path = self.root.join(src);
        let dest_path = self.root.join(dest);

        if !src_path.exists() {
            return Err(StorageError::Error(format!(
                "Source path does not exist: {}",
                src_path.display()
            )));
        }

        for entry in WalkDir::new(&src_path) {
            let entry = entry
                .map_err(|e| StorageError::Error(format!("Unable to read directory: {}", e)))?;
            let relative_path = entry
                .path()
                .strip_prefix(&src_path)
                .map_err(|e| StorageError::Error(format!("Unable to strip prefix: {}", e)))?;
            let dest_file_path = dest_path.join(relative_path);

            if entry.file_type().is_file() {
                if let Some(parent) = dest_file_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        StorageError::Error(format!("Unable to create directory: {}", e))
                    })?;
                }

                fs::copy(entry.path(), &dest_file_path)
                    .map_err(|e| StorageError::Error(format!("Unable to copy file: {}", e)))?;
            }
        }

        Ok(true)
    }

    fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
        let full_path = self.root.join(path);

        if !full_path.exists() {
            return Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )));
        }

        fs::remove_file(&full_path)
            .map_err(|e| StorageError::Error(format!("Unable to delete file: {}", e)))?;

        Ok(true)
    }

    fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
        let full_path = self.root.join(path);

        if !full_path.exists() {
            return Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )));
        }

        for entry in WalkDir::new(&full_path) {
            let entry = entry
                .map_err(|e| StorageError::Error(format!("Unable to read directory: {}", e)))?;
            if entry.file_type().is_file() {
                fs::remove_file(entry.path())
                    .map_err(|e| StorageError::Error(format!("Unable to delete file: {}", e)))?;
            }
        }

        Ok(true)
    }
}

pub struct LocalFSStorageClient {
    client: LocalStorageClient,
}

impl FileSystem<LocalStorageClient> for LocalFSStorageClient {
    fn client(&self) -> &LocalStorageClient {
        &self.client
    }

    fn new(root: String) -> Self {
        Self {
            client: LocalStorageClient::new(root),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_local_storage_client() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap().to_string();
        let client = LocalStorageClient::new(root.clone());

        // Create a test file
        let test_file_path = temp_dir.path().join("test_file.txt");
        let mut test_file = File::create(&test_file_path).unwrap();
        writeln!(test_file, "Hello, world!").unwrap();

        // Test get_object
        let local_path = temp_dir.path().join("local_test_file.txt");
        client
            .get_object(local_path.to_str().unwrap(), "test_file.txt")
            .unwrap();
        assert!(local_path.exists());

        // Test generate_presigned_url
        let url = client
            .generate_presigned_url("test_file.txt", 3600)
            .unwrap();
        assert_eq!(url, test_file_path.to_str().unwrap());

        // Test upload_file_in_chunks
        let upload_path = temp_dir.path().join("upload_test_file.txt");
        client
            .upload_file_in_chunks(&test_file_path, &upload_path, None)
            .unwrap();
        assert!(upload_path.exists());

        // Test find
        let files = client.find("").unwrap();
        assert_eq!(files.len(), 3); // 3 files: test_file.txt, local_test_file.txt, upload_test_file.txt

        // Test find_info
        let files_info = client.find_info("").unwrap();
        assert_eq!(files_info.len(), 3);

        // Test copy_object
        let copy_path = temp_dir.path().join("copy_test_file.txt");
        client
            .copy_object("test_file.txt", copy_path.to_str().unwrap())
            .unwrap();
        assert!(copy_path.exists());

        // Test copy_objects
        let copy_dir_path = temp_dir.path().join("copy_dir");
        client
            .copy_objects("", copy_dir_path.to_str().unwrap())
            .unwrap();
        assert!(copy_dir_path.exists());

        // Test delete_object
        client.delete_object("test_file.txt").unwrap();
        assert!(!test_file_path.exists());

        // Test delete_objects
        client.delete_objects("").unwrap();
        assert!(temp_dir.path().read_dir().unwrap().next().is_none());
    }
}
