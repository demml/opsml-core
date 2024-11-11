import asyncio
from opsml_storage import GCSFSStorageClient
from pathlib import Path


def main():
    # Create a storage client
    storage_client = GCSFSStorageClient(bucket="opsml-storage-integration")

    # Find all the files in the path
    files = storage_client.find(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1")
    )

    # stream = storage_client.iterfile(
    #    path=Path(
    #        "OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/quantized-model/config.json"
    #    )
    # )

    storage_client.put(
        lpath=Path("tests/assets"),
        rpath=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/check"),
    )


if __name__ == "__main__":
    main()
