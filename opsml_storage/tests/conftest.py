import pytest
import os
from opsml_storage import GCSFSStorageClient


@pytest.fixture
def gcs_storage_client():
    return GCSFSStorageClient(bucket=os.environ["GCP_BUCKET_NAME"])
