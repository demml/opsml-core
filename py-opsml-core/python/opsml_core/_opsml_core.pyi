from enum import Enum
from typing import Literal, List, Optional

class ClientType(Enum):
    GCS: Literal["GCS"]

class OpsmlStorageClient:
    def __init__(self, client_type: ClientType, bucket_name: str):
        """Initialize the storage client.

        Args:
            client_type:
                The type of the storage client.
            bucket_name:
                The name of the bucket. Currently only GCS is supported.
        """
        ...

    async def find(self, path: str) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of file paths.
        """
        ...

    async def create_resumable_upload_session(
        self, path: str, chunk_size: int, total_size: int
    ) -> None:
        """Creates a resumable upload session.

        Args:
            path:
                The path to upload the file to.
            chunk_size:
                The size of each chunk in bytes.
            total_size:
                The total size of the file in bytes.

        Returns:
            None
        """
        ...

    async def upload_chunk(self, data: bytes) -> Optional[str]:
        """Uploads a chunk of data.

        Args:
            data:
                The data to upload.

        Returns:
            If the upload is complete, the path to the uploaded file. Otherwise, None.
        """
        ...
