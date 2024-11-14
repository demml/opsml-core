from pathlib import Path
from opsml_storage_s3 import PyS3FSStorageClient
import uuid


def test_aws_storage_client(
    tmp_path: Path, aws_storage_client: PyS3FSStorageClient
) -> None:
    # generate random directory

    lpath = Path("tests/assets/cats.jpg")
    rpath_dir = Path(uuid.uuid4().hex)
    rpath = rpath_dir / "cats.jpg"

    get_lpath = Path(tmp_path / "tests/assets/empty.cats.jpg")

    try:
        if aws_storage_client.exists(rpath_dir):
            aws_storage_client.rm(rpath_dir, recursive=True)
        assert not aws_storage_client.exists(rpath_dir)

        # put
        aws_storage_client.put(lpath, rpath)
        assert aws_storage_client.exists(rpath)
        rpath_nested = rpath.parent / "nested/really/deep/cats-2.jpg"
        aws_storage_client.put(lpath, rpath_nested)

        # generate_presigned_url
        # get bucket
        path = aws_storage_client.generate_presigned_url(rpath, 1)
        assert path is not None

        ## ls
        assert len(aws_storage_client.find()) >= 1
        assert len(aws_storage_client.find(rpath_nested.parent)) >= 1

        # find
        assert aws_storage_client.find(rpath_dir) == [
            rpath.as_posix(),
            rpath_nested.as_posix(),
        ]

        ## get
        get_lpath = tmp_path / "cats.jpg"
        aws_storage_client.get(get_lpath, rpath)
        assert get_lpath.exists()

        ## rm
        aws_storage_client.rm(rpath)
        assert not aws_storage_client.exists(rpath)
    finally:
        if aws_storage_client.exists(rpath_dir):
            aws_storage_client.rm(rpath_dir, True)
