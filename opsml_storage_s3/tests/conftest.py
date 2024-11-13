import pytest
import os
from opsml_storage_s3 import PyS3FSStorageClient


@pytest.fixture
def s3_storage_client() -> PyS3FSStorageClient:
    return PyS3FSStorageClient(bucket=os.environ["CLOUD_BUCKET_NAME"])
