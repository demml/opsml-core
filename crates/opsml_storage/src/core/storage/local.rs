use crate::core::storage::base::get_files;
use crate::core::storage::base::PathExt;
use crate::core::storage::base::{FileInfo, FileSystem, StorageClient};
use crate::core::utils::error::StorageError;
use async_trait::async_trait;
use aws_smithy_types::byte_stream::{ByteStream, Length};
use opsml_settings::config::{OpsmlStorageSettings, StorageType};
use pyo3::prelude::*;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

// left off here
// removed multiupload part and implemented put on each storage client
// need to fix up http client
// - method for creating resumable upload
// - method for creating uploader from resumable upload
// - method for uploading part (special handling for local storage, or do we just use the same method?)

pub struct LocalMultiPartUpload {
    pub file: fs::File,
    pub path: String,
}

impl LocalMultiPartUpload {
    pub async fn new(path: &Path) -> Self {
        // check if path parent exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))
                .unwrap();
        }

        // create a new file
        let file = fs::File::create(path)
            .map_err(|e| StorageError::Error(format!("Unable to create file: {}", e)))
            .unwrap();

        Self {
            file,
            path: path.to_str().unwrap().to_string(),
        }
    }

    pub async fn get_next_chunk(
        &self,
        path: &Path,
        chunk_size: u64,
        chunk_index: u64,
        this_chunk_size: u64,
    ) -> Result<ByteStream, StorageError> {
        let stream = ByteStream::read_from()
            .path(path)
            .offset(chunk_index * chunk_size)
            .length(Length::Exact(this_chunk_size))
            .build()
            .await
            .map_err(|e| StorageError::Error(format!("Failed to get next chunk: {}", e)))?;

        Ok(stream)
    }

    pub async fn upload_part(&mut self, chunk: bytes::Bytes) -> Result<bool, StorageError> {
        self.file
            .write_all(&chunk)
            .map_err(|e| StorageError::Error(format!("Unable to write to file: {}", e)))?;

        Ok(true)
    }

    pub async fn complete_upload(&mut self) -> Result<(), StorageError> {
        Ok(())
    }
}

pub struct LocalStorageClient {
    pub bucket: PathBuf,
}

#[async_trait]
impl StorageClient for LocalStorageClient {
    fn storage_type(&self) -> StorageType {
        StorageType::Local
    }
    async fn bucket(&self) -> &str {
        self.bucket.to_str().unwrap()
    }

    async fn new(settings: &OpsmlStorageSettings) -> Result<Self, StorageError> {
        let bucket = PathBuf::from(settings.storage_uri.as_str());

        // bucket should be a dir. Check if it exists. If not, create it
        if !bucket.exists() {
            fs::create_dir_all(&bucket)
                .map_err(|e| {
                    StorageError::Error(format!("Unable to create bucket directory: {}", e))
                })
                .unwrap();
        }

        Ok(Self { bucket })
    }

    async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
        let src_path = self.bucket.join(rpath);
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

    async fn generate_presigned_url(
        &self,
        path: &str,
        _expiration: u64,
    ) -> Result<String, StorageError> {
        let full_path = self.bucket.join(path);
        if full_path.exists() {
            Ok(full_path.to_str().unwrap().to_string())
        } else {
            Err(StorageError::Error(format!(
                "Path does not exist: {}",
                full_path.display()
            )))
        }
    }

    async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let full_path = self.bucket.join(path);
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

        // remove the bucket name and any following slashes
        let bucket = self.bucket.to_str().unwrap();
        let files = files
            .iter()
            .map(|f| f.strip_prefix(bucket).unwrap_or(f))
            .map(|f| f.strip_prefix("/").unwrap_or(f))
            .map(|f| f.to_string())
            .collect();

        Ok(files)
    }

    async fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
        let full_path = self.bucket.join(path);
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

    async fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
        let src_path = self.bucket.join(src);
        let dest_path = self.bucket.join(dest);

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

    async fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
        let src_path = self.bucket.join(src);
        let dest_path = self.bucket.join(dest);

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

    async fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
        let full_path = self.bucket.join(path);

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

    async fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
        let full_path = self.bucket.join(path);

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

impl LocalStorageClient {
    pub async fn create_multipart_upload(&self, path: &str) -> Result<String, StorageError> {
        Ok(path.to_string())
    }

    async fn upload_file(&self, lpath: &Path, rpath: &Path) -> Result<(), StorageError> {
        let dest_path = self.bucket.join(rpath);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))?;
        }

        fs::copy(lpath, &dest_path)
            .map_err(|e| StorageError::Error(format!("Unable to copy file: {}", e)))?;

        Ok(())
    }
}

pub struct LocalFSStorageClient {
    client: LocalStorageClient,
}

#[async_trait]
impl FileSystem<LocalStorageClient> for LocalFSStorageClient {
    fn name(&self) -> &str {
        "LocalFSStorageClient"
    }

    fn client(&self) -> &LocalStorageClient {
        &self.client
    }

    async fn new(settings: &OpsmlStorageSettings) -> Self {
        Self {
            client: LocalStorageClient::new(settings).await.unwrap(),
        }
    }
}

impl LocalFSStorageClient {
    pub async fn put(
        &self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        let stripped_lpath = lpath.strip_path(self.client().bucket().await);
        let stripped_rpath = rpath.strip_path(self.client().bucket().await);

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
                let stripped_file_path = file.strip_path(self.client().bucket().await);

                let relative_path = file.relative_path(&stripped_lpath_clone)?;
                let remote_path = stripped_rpath_clone.join(relative_path);

                self.client()
                    .upload_file(&stripped_file_path, &remote_path)
                    .await?;
            }

            Ok(())
        } else {
            self.client()
                .upload_file(&stripped_lpath, &stripped_rpath)
                .await?;
            Ok(())
        }
    }
}

#[pyclass]
pub struct PyLocalFSStorageClient {
    client: LocalStorageClient,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyLocalFSStorageClient {
    #[new]
    fn new(settings: &OpsmlStorageSettings) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = rt
            .block_on(async { LocalStorageClient::new(settings).await })
            .unwrap();
        Self {
            client,
            runtime: rt,
        }
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find_info(&self, path: PathBuf) -> Result<Vec<FileInfo>, StorageError> {
        let stripped_path = path.strip_path(self.client.bucket.to_str().unwrap());

        self.runtime
            .block_on(async { self.client.find_info(stripped_path.to_str().unwrap()).await })
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find(&self, path: PathBuf) -> Result<Vec<String>, StorageError> {
        let stripped_path = path.strip_path(self.client.bucket.to_str().unwrap());
        let files = self
            .runtime
            .block_on(async { self.client.find(stripped_path.to_str().unwrap()).await })?;

        // attempt to remove the bucket name from the path
        let stripped_files = files
            .iter()
            .map(|f| {
                f.strip_prefix(self.client.bucket.to_str().unwrap())
                    .unwrap_or(f)
                    .to_string()
            })
            .collect();

        Ok(stripped_files)
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
        // strip the paths
        let stripped_rpath = rpath.strip_path(self.client.bucket.to_str().unwrap());
        let stripped_lpath = lpath.strip_path(self.client.bucket.to_str().unwrap());

        self.runtime.block_on(async {
            if recursive {
                let stripped_lpath_clone = stripped_lpath.clone();

                // list all objects in the path
                let objects = self.client.find(stripped_rpath.to_str().unwrap()).await?;

                // iterate over each object and get it
                for obj in objects {
                    let file_path = Path::new(obj.as_str());
                    let stripped_path = file_path.strip_path(self.client.bucket.to_str().unwrap());
                    let relative_path = file_path.relative_path(&stripped_rpath)?;
                    let local_path = stripped_lpath_clone.join(relative_path);

                    self.client
                        .get_object(
                            local_path.to_str().unwrap(),
                            stripped_path.to_str().unwrap(),
                        )
                        .await?;
                }
            } else {
                self.client
                    .get_object(
                        stripped_lpath.to_str().unwrap(),
                        stripped_rpath.to_str().unwrap(),
                    )
                    .await?;
            }

            Ok(())
        })
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    fn put(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
        let stripped_lpath = lpath.strip_path(self.client.bucket.to_str().unwrap());
        let stripped_rpath = rpath.strip_path(self.client.bucket.to_str().unwrap());

        self.runtime.block_on(async {
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
                    let stripped_file_path = file.strip_path(self.client.bucket.to_str().unwrap());

                    let relative_path = file.relative_path(&stripped_lpath_clone)?;
                    let remote_path = stripped_rpath_clone.join(relative_path);

                    self.client
                        .upload_file(&stripped_file_path, &remote_path)
                        .await?;
                }

                Ok(())
            } else {
                self.client
                    .upload_file(&stripped_lpath, &stripped_rpath)
                    .await?;
                Ok(())
            }
        })
    }

    #[pyo3(signature = (src, dest, recursive = false))]
    fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> Result<(), StorageError> {
        let stripped_src = src.strip_path(self.client.bucket.to_str().unwrap());
        let stripped_dest = dest.strip_path(self.client.bucket.to_str().unwrap());

        self.runtime.block_on(async {
            if recursive {
                self.client
                    .copy_objects(
                        stripped_src.to_str().unwrap(),
                        stripped_dest.to_str().unwrap(),
                    )
                    .await?;
            } else {
                self.client
                    .copy_object(
                        stripped_src.to_str().unwrap(),
                        stripped_dest.to_str().unwrap(),
                    )
                    .await?;
            }

            Ok(())
        })
    }

    #[pyo3(signature = (path, recursive = false))]
    fn rm(&self, path: PathBuf, recursive: bool) -> Result<(), StorageError> {
        let stripped_path = path.strip_path(self.client.bucket.to_str().unwrap());

        self.runtime.block_on(async {
            if recursive {
                self.client
                    .delete_objects(stripped_path.to_str().unwrap())
                    .await?;
            } else {
                self.client
                    .delete_object(stripped_path.to_str().unwrap())
                    .await?;
            }

            Ok(())
        })
    }

    fn exists(&self, path: PathBuf) -> Result<bool, StorageError> {
        let stripped_path = path.strip_path(self.client.bucket.to_str().unwrap());
        let objects = self
            .runtime
            .block_on(async { self.client.find(stripped_path.to_str().unwrap()).await });

        // if error, return false
        if objects.is_err() {
            Ok(false)
        } else {
            let objects = objects?;
            Ok(!objects.is_empty())
        }
    }

    #[pyo3(signature = (path, expiration = 600))]
    fn generate_presigned_url(
        &self,
        path: PathBuf,
        expiration: u64,
    ) -> Result<String, StorageError> {
        let stripped_path = path.strip_path(self.client.bucket.to_str().unwrap());
        self.runtime.block_on(async {
            self.client
                .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
                .await
        })
    }

    fn delete_bucket(&self) -> Result<(), StorageError> {
        fs::remove_dir_all(&self.client.bucket)
            .map_err(|e| StorageError::Error(format!("Unable to delete bucket: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::base::get_files;
    use opsml_settings::config::OpsmlConfig;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    const CHUNK_SIZE: u64 = 1024 * 256;

    fn get_settings() -> OpsmlStorageSettings {
        let bucket = std::env::var("CLOUD_BUCKET_NAME")
            .unwrap_or_else(|_| "opsml-storage-integration".to_string());

        let config = OpsmlConfig::new();
        let mut settings = config.storage_settings();
        settings.storage_uri = bucket;

        settings
    }

    pub fn create_file(name: &str, chunk_size: &u64) {
        let mut file = File::create(name).expect("Could not create sample file.");

        while file.metadata().unwrap().len() <= chunk_size * 2 {
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(256)
                .map(char::from)
                .collect();
            let return_string: String = "\n".to_string();
            file.write_all(rand_string.as_ref())
                .expect("Error writing to file.");
            file.write_all(return_string.as_ref())
                .expect("Error writing to file.");
        }
    }

    pub fn create_nested_data(chunk_size: &u64) -> String {
        let rand_name = uuid::Uuid::new_v4().to_string();

        // create a temporary directory
        let dir_name = format!("temp_test_dir_{}", &rand_name);
        let dir = Path::new(&dir_name);

        if !dir.exists() {
            std::fs::create_dir_all(dir).unwrap();
        }
        // random file name with uuid
        let key = format!("{}/temp_test_file_{}.txt", &dir_name, &rand_name);
        create_file(&key, chunk_size);

        // created nested directories
        let dir = Path::new(&dir_name);
        let nested_dir = dir.join("nested_dir");
        let nested_dir_path = nested_dir.to_str().unwrap();

        if !nested_dir.exists() {
            std::fs::create_dir_all(nested_dir.clone()).unwrap();
        }

        // random file name with uuid
        let key = format!("{}/temp_test_file_{}.txt", &nested_dir_path, &rand_name);
        create_file(&key, chunk_size);

        dir_name
    }

    fn create_single_file(chunk_size: &u64) -> String {
        let rand_name = uuid::Uuid::new_v4().to_string();

        // create a temporary directory
        let dir_name = format!("temp_test_dir_{}", &rand_name);
        let dir = Path::new(&dir_name);

        if !dir.exists() {
            std::fs::create_dir_all(dir).unwrap();
        }

        // random file name with uuid
        let key = format!("{}/temp_test_file_{}.txt", &dir_name, &rand_name);
        create_file(&key, chunk_size);

        key
    }

    #[test]
    fn test_local_storage_client_creds() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap().to_string();
        let mut settings = get_settings();
        settings.storage_uri = root.clone();

        let _client = LocalStorageClient::new(&settings);
    }

    #[tokio::test]
    async fn test_local_storage_client_get_object() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap().to_string();

        let mut settings = get_settings();
        settings.storage_uri = root.clone();

        let client = LocalStorageClient::new(&settings).await.unwrap();

        // should fail since there are no suffixes
        let result = client.get_object("local_path", "remote_path").await;
        assert!(result.is_err()); // Assuming the object does not exist
    }

    #[tokio::test]
    async fn test_local_storage_client_put() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_str().unwrap().to_string();

        let mut settings = get_settings();
        settings.storage_uri = root.clone();
        let client = LocalFSStorageClient::new(&settings).await;

        //
        let dirname = create_nested_data(&CHUNK_SIZE);

        let lpath = Path::new(&dirname);
        let rpath = Path::new(&dirname);

        // put the file
        client.put(lpath, rpath, true).await.unwrap();

        // check if the file exists
        let exists = client.exists(rpath).await.unwrap();
        assert!(exists);

        // list all files
        let files = client.find(rpath).await.unwrap();
        assert_eq!(files.len(), 2);

        // list files with info
        let files = client.find_info(rpath).await.unwrap();
        assert_eq!(files.len(), 2);

        // download the files
        let new_path = uuid::Uuid::new_v4().to_string();
        let new_path = Path::new(&new_path);

        client.get(new_path, rpath, true).await.unwrap();

        // check if the files are the same
        let files = get_files(rpath).unwrap();
        let new_files = get_files(new_path).unwrap();

        assert_eq!(files.len(), new_files.len());

        // copy the files
        // create a new path
        let copy_path = uuid::Uuid::new_v4().to_string();
        let copy_path = Path::new(&copy_path);
        client.copy(rpath, copy_path, true).await.unwrap();
        let files = client.find(copy_path).await.unwrap();
        assert_eq!(files.len(), 2);

        // cleanup
        std::fs::remove_dir_all(&dirname).unwrap();
        std::fs::remove_dir_all(new_path).unwrap();

        client.rm(rpath, true).await.unwrap();
        client.rm(copy_path, true).await.unwrap();

        // check if the file exists
        let exists = client.exists(rpath).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_local_storage_client_generate_presigned_url() {
        let settings = get_settings();
        let client = LocalFSStorageClient::new(&settings).await;

        // create file
        let key = create_single_file(&CHUNK_SIZE);
        let path = Path::new(&key);

        // put the file
        client.put(path, path, false).await.unwrap();

        // generate presigned url
        let url = client.generate_presigned_url(path, 3600).await.unwrap();
        assert!(!url.is_empty());

        // cleanup
        client.rm(path, false).await.unwrap();
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn test_local_large_file_upload() {
        let settings = get_settings();
        let client = LocalFSStorageClient::new(&settings).await;

        // create file
        let chunk_size = 1024 * 1024 * 5; // 5MB
        let key = create_single_file(&chunk_size);
        let path = Path::new(&key);

        // put the file
        client.put(path, path, false).await.unwrap();

        // get the file info
        let info = client.find_info(path).await.unwrap();
        assert_eq!(info.len(), 1);

        // get item and assert it's at least the size of the file
        let item = info.first().unwrap();
        assert!(item.size >= 1024 * 1024 * 10);

        // cleanup
        client.rm(path, false).await.unwrap();
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
