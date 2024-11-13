from pathlib import Path
from opsml_storage_s3 import PyS3FSStorageClient


def test_aws_storage_client(
    tmp_path: Path, aws_storage_client: PyS3FSStorageClient
) -> None:
    lpath = Path("tests/assets/cats.jpg")
    rpath_dir = Path("test_dir")
    rpath = rpath_dir / "cats.jpg"

    get_lpath = Path(tmp_path / "tests/assets/empty.cats.jpg")

    try:
        if aws_storage_client.exists(rpath_dir):
            aws_storage_client.rm(rpath_dir)
        assert not aws_storage_client.exists(rpath_dir)

        # put
    # aws_storage_client.put(lpath, rpath)
    # assert aws_storage_client.exists(rpath)
    # rpath_nested = rpath.parent / "nested/really/deep/cats-2.jpg"
    # aws_storage_client.put(lpath, rpath_nested)

    ## generate_presigned_url
    ## get bucket
    # blob_path = rpath.relative_to(aws_s3_bucket)
    # path = aws_storage_client.generate_presigned_url(blob_path, 1)
    # assert path is not None

    ## ls
    # assert len(aws_storage_client.ls(aws_s3_bucket)) >= 1
    # assert len(aws_storage_client.ls(rpath_nested.parent)) >= 1

    ## find
    # assert aws_storage_client.find(rpath_dir) == [rpath, rpath_nested]

    ## get
    # get_lpath = tmp_path / "cats.jpg"
    # aws_storage_client.get(rpath, get_lpath)
    # assert get_lpath.exists()

    ## iterfile
    # for f in aws_storage_client.iterfile(rpath, 1000):
    #    _ = lpath.read_bytes()

    ## rm
    # aws_storage_client.rm(rpath)
    # assert not aws_storage_client.exists(rpath)
    finally:
        pass
        # if aws_storage_client.exists(rpath_dir):
        # aws_storage_client.rm(rpath_dir)
