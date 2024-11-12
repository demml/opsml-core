from pathlib import Path
from typing import Iterable, List

class GCSFSStorageClient:
    def __init__(self, bucket: str):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the gcs bucket.
        """

    def find(self, path: Path) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of file paths.
        """

    def iterfile(self, path: Path) -> Iterable[bytes]:
        """Returns an iterator over all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            An iterator over file paths.
        """

    def put(self, lpath: Path, rpath: Path):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
        """

    def copy(self, src: Path, dest: Path, recursive: bool = False):
        """Copy the data from the source to the destination.

        Args:
            src:
                The source path.
            dest:
                The destination path.
            recursive:
                Whether to copy recursively.
        """

    def rm(self, path: Path, recursive: bool = False):
        """Remove the data from the source.

        Args:
            path:
                The source path.
            recursive:
                Whether to remove recursively.
        """

    def exists(self, path: Path) -> bool:
        """Check if the path exists.

        Args:
            path:
                path to check.
        """

    def get(self, lpath: Path, rpath: Path, recursive: bool = False) -> None:
        """Get the data from the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
            recursive:
                Whether to get recursively.
        """

    def generate_presigned_url(self, path: Path, expiration: int = 600) -> str:
        """Generate a signed URL for the path.

        Args:
            path:
                The path to the file.
            expiration:
                The expiration time in seconds.
        """
