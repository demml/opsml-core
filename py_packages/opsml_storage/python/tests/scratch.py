import asyncio
from opsml_storage import GCSFSStorageClient
from pathlib import Path


def main():
    # Create a storage client
    storage_client = GCSFSStorageClient(bucket="opsml-dev")

    # Find all the files in the path
    files = storage_client.find(
        path=Path("OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1")
    )

    stream = storage_client.iterfile(
        path=Path(
            "OPSML_MODEL_REGISTRY/mlops/test-model/v0.0.1/quantized-model/config.json"
        )
    )

    # write the stream to a file
    with open("config.json", "wb") as f:
        for chunk in stream:
            print(chunk)


if __name__ == "__main__":
    main()
