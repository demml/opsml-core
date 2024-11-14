import pytest
import os
from opsml_storage_gcs import PyGCSFSStorageClient


@pytest.fixture
def aws_storage_client() -> PyGCSFSStorageClient:
    return PyGCSFSStorageClient(bucket=os.environ["CLOUD_BUCKET_NAME"])
