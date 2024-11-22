import pytest
import os
from opsml_storage_local import PyLocalFSStorageClient


@pytest.fixture
def storage_client() -> PyLocalFSStorageClient:
    return PyLocalFSStorageClient(
        bucket=os.environ.get("CLOUD_BUCKET_NAME", "opsml-storage-integration")
    )
