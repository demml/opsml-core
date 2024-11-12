from opsml_storage import GCSFSStorageClient
from pathlib import Path
import shutil


def test_storage_methods(gcs_storage_client: GCSFSStorageClient):
  

    # Find all the files in the path
    _files = gcs_storage_client.find(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
    )

    gcs_storage_client.put(
        lpath=Path("tests/assets"),
        rpath=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
    )

    assert (
        len(
            gcs_storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
            )
        )
        > 0
    ), "No files found"

    gcs_storage_client.copy(
        src=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
        dest=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    assert (
        len(
            gcs_storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy")
            )
        )
        > 0
    ), "No files found"

    assert gcs_storage_client.exists(
        path=Path(
            "OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy/opsml_logo.png"
        )
    ), "File not found"

    gcs_storage_client.get(
        lpath=Path("tests/assets/new"),
        rpath=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    url = gcs_storage_client.generate_presigned_url(
        Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy/opsml_logo.png")
    )

    assert url, "URL not generated"

    gcs_storage_client.rm(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
        recursive=True,
    )

    assert (
        len(
            gcs_storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
            )
        )
        == 0
    ), "Files present"

    gcs_storage_client.rm(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    assert (
        len(
            gcs_storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy")
            )
        )
        == 0
    ), "Files present"

    shutil.rmtree("tests/assets/new", ignore_errors=True)


def test_storage_client()