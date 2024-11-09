from opsml_storage import GCSFSStorageClient
from pathlib import Path

if __name__ == "__main__":
    # Create a storage client
    storage_client = GCSFSStorageClient(bucket_name="my-bucket")

    # Find all the files in the path
    files = storage_client.find(path=Path("path/to/search"))
    print(files)
