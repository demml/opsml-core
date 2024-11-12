import pytest
import os


@pytest.fixture
def gcs_bucket_name():
    return os.getenv("GCP_BUCKET_NAME")
