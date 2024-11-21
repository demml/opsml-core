if __name__ == "__main__":
    from pathlib import Path
    import os
    import uuid

    from opsml_storage_local import PyHttpFSStorageClient, OpsmlConfig

    os.environ["OPSML_TRACKING_URI"] = "http://localhost:3000"

    config = OpsmlConfig()
    storage_client = PyHttpFSStorageClient(config.storage_settings())

    path = Path("inkscape")

    files = storage_client.find(path)

    print(files)

    info = storage_client.find_info(path)

    print(info)

    # kwargs = {
    #    "base_url": "http://localhost:3000",
    #    "path_prefix": "opsml",
    # }
    # storage_uri = "http://localhost:3000"
    # storage_type = StorageType.Google
    # settings = StorageSettings(storage_uri, True, storage_type, kwargs)
#
# client = PyHttpFSStorageClient(settings)
#
# lpath = Path("tests/assets/cats.jpg")
# rpath_dir = Path(uuid.uuid4().hex)
# rpath = rpath_dir / "cats.jpg"
#
# client.put(lpath, rpath)
