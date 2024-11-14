import pytest
import os
from opsml_storage_gcs import PyGCSFSStorageClient


@pytest.fixture
def storage_client() -> PyGCSFSStorageClient:
    return PyGCSFSStorageClient(
        bucket=os.environ.get("CLOUD_BUCKET_NAME", "opsml-storage-integration")
    )
