if __name__ == "__main__":
    from pathlib import Path
    import uuid
    from opsml_storage_local import PyHttpFSStorageClient, StorageType, StorageSettings

    kwargs = {
        "base_url": "http://localhost:3000",
        "path_prefix": "opsml",
    }
    storage_uri = "http://localhost:3000"
    storage_type = StorageType.Google
    settings = StorageSettings(storage_uri, True, storage_type, kwargs)

    client = PyHttpFSStorageClient(settings)

    lpath = Path("tests/assets/cats.jpg")
    rpath_dir = Path(uuid.uuid4().hex)
    rpath = rpath_dir / "cats.jpg"

    client.put(lpath, rpath)
