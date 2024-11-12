import pytest
import os
from opsml_storage import GCSFSStorageClient


@pytest.fixture
def gcs_storage_client():
    value = os.environ["GOOGLE_APPLICATION_CREDENTIALS"]
    assert value, "GOOGLE_APPLICATION_CREDENTIALS not set"
    return GCSFSStorageClient(bucket=os.environ["GCS_BUCKET_NAME"])
