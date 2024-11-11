from opsml_storage import GCSFSStorageClient
from pathlib import Path
import shutil


def main():
    # Create a storage client
    storage_client = GCSFSStorageClient(bucket="opsml-storage-integration")

    # Find all the files in the path
    files = storage_client.find(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
    )

    storage_client.put(
        lpath=Path("tests/assets"),
        rpath=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
    )

    assert (
        len(
            storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
            )
        )
        > 0
    ), "No files found"

    storage_client.copy(
        src=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
        dest=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    assert (
        len(
            storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy")
            )
        )
        > 0
    ), "No files found"

    assert storage_client.exists(
        path=Path(
            "OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy/opsml_logo.png"
        )
    ), "File not found"

    storage_client.get(
        lpath=Path("tests/assets/new"),
        rpath=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    url = storage_client.generate_presigned_url(
        Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy/opsml_logo.png")
    )

    assert url, "URL not generated"

    storage_client.rm(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
        recursive=True,
    )

    assert (
        len(
            storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check")
            )
        )
        == 0
    ), "Files present"

    storage_client.rm(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy"),
        recursive=True,
    )

    assert (
        len(
            storage_client.find(
                path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check-copy")
            )
        )
        == 0
    ), "Files present"

    shutil.rmtree("tests/assets/new", ignore_errors=True)


if __name__ == "__main__":
    main()
