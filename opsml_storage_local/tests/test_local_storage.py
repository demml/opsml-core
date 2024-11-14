from pathlib import Path
from opsml_storage_local import PyLocalFSStorageClient
import uuid
import shutil


def test_storage_client(tmp_path: Path, storage_client: PyLocalFSStorageClient) -> None:
    # generate random directory

    lpath = Path("tests/assets/cats.jpg")
    rpath_dir = Path(uuid.uuid4().hex)
    rpath = rpath_dir / "cats.jpg"

    get_lpath = Path(tmp_path / "tests/assets/empty.cats.jpg")

    try:
        if storage_client.exists(rpath_dir):
            storage_client.rm(rpath_dir, recursive=True)
        assert not storage_client.exists(rpath_dir)

        # put
        storage_client.put(lpath, rpath)
        assert storage_client.exists(rpath)
        rpath_nested = rpath.parent / "nested/really/deep/cats-2.jpg"
        storage_client.put(lpath, rpath_nested)

        # generate_presigned_url
        # get bucket
        path = storage_client.generate_presigned_url(rpath, 1)
        assert path is not None

        ## ls
        assert len(storage_client.find()) >= 1
        assert len(storage_client.find(rpath_nested.parent)) >= 1

        # find
        assert storage_client.find(rpath_dir) == [
            rpath.as_posix(),
            rpath_nested.as_posix(),
        ]

        ## get
        get_lpath = tmp_path / "cats.jpg"
        storage_client.get(get_lpath, rpath)
        assert get_lpath.exists()

        ## rm
        storage_client.rm(rpath)
        assert not storage_client.exists(rpath)
    finally:
        if storage_client.exists(rpath_dir):
            storage_client.rm(rpath_dir, True)
        storage_client.delete_bucket()
