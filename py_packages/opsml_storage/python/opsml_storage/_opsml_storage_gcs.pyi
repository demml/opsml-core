from typing import List
from pathlib import Path

class GCSFSStorageClient:
    def __init__(self, bucket_name: str):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the gcs bucket.
        """
        ...

    async def find(self, path: Path) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of file paths.
        """
        ...
