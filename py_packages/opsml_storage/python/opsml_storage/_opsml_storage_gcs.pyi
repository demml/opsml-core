from typing import List, Iterable
from pathlib import Path

class GCSFSStorageClient:
    def __init__(self, bucket: str):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the gcs bucket.
        """
        ...

    def find(self, path: Path) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of file paths.
        """
        ...

    def iterfile(self, path: Path) -> Iterable[bytes]:
        """Returns an iterator over all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            An iterator over file paths.
        """
        ...

    def put(self, lpath: Path, rpath: Path):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
        """
        ...
