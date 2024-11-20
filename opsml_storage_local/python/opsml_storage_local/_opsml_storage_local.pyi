from pathlib import Path
from typing import List, Optional, Dict
from enum import Enum

class StorageType(str, Enum):
    Google = "google"
    AWS = "aws"
    Local = "local"

class StorageSettings:
    def __init__(
        self,
        storage_uri: str,
        using_client: bool,
        storage_type: StorageType,
        kwargs: Dict[str, str],
    ) -> None:
        """Initialize the storage settings.

        Args:
            storage_uri:
                The storage URI.
            using_client:
                Whether to use the client.
            kwargs:
                Additional arguments.
        """

    @property
    def storage_uri(self) -> str:
        """The storage URI."""

    @property
    def using_client(self) -> bool:
        """Whether to use the client."""

    @property
    def kwargs(self) -> Dict[str, str]:
        """Additional arguments."""

    @property
    def storage_type(self) -> StorageType:
        """The storage type."""

class FileInfo:
    @property
    def name(self) -> str:
        """The name of the file."""

    @property
    def size(self) -> int:
        """The size of the file."""

    @property
    def object_type(self) -> str:
        """The type of the object."""

    @property
    def created(self) -> str:
        """The creation time of the file."""

    @property
    def suffix(self) -> str:
        """The suffix of the file."""

class PyLocalFSStorageClient:
    def __init__(self, settings: StorageSettings):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the s3 bucket.
        """

    def find(self, path: Optional[Path] = None) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.
        """

    def find_info(self, path: Optional[Path] = None) -> List[FileInfo]:
        """Returns all the files in the path with additional information.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of FileInfo objects.
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

    def put(self, lpath: Path, rpath: Path, recursive: bool = False):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
            recursive:
                Whether to put recursively. lpath and rpath must be directories
        """

    def copy(self, src: Path, dest: Path, recursive: bool = False):
        """Copy the data from the source to the destination. This is an "in-bucket" copy.

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

        Returns:
            True if the path exists, False otherwise.
        """

    def generate_presigned_url(self, path: Path, expiration: int = 600) -> str:
        """Generate a signed URL for the path.

        Args:
            path:
                The path to the file.
            expiration:
                The expiration time in seconds.

        Returns:
            The signed URL.
        """

    def delete_bucket(self) -> None:
        """Delete the bucket."""

class PyHttpFSStorageClient:
    def __init__(self, settings: StorageSettings):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the s3 bucket.
        """

    def find(self, path: Optional[Path] = None) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.
        """

    def find_info(self, path: Optional[Path] = None) -> List[FileInfo]:
        """Returns all the files in the path with additional information.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of FileInfo objects.
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

    def put(self, lpath: Path, rpath: Path, recursive: bool = False):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
            recursive:
                Whether to put recursively. lpath and rpath must be directories
        """

    def copy(self, src: Path, dest: Path, recursive: bool = False):
        """Copy the data from the source to the destination. This is an "in-bucket" copy.

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

        Returns:
            True if the path exists, False otherwise.
        """

    def generate_presigned_url(self, path: Path, expiration: int = 600) -> str:
        """Generate a signed URL for the path.

        Args:
            path:
                The path to the file.
            expiration:
                The expiration time in seconds.

        Returns:
            The signed URL.
        """

    def delete_bucket(self) -> None:
        """Delete the bucket."""

class PyS3FSStorageClient:
    def __init__(self, settings: StorageSettings):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the s3 bucket.
        """

    def find(self, path: Optional[Path] = None) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.
        """

    def find_info(self, path: Optional[Path] = None) -> List[FileInfo]:
        """Returns all the files in the path with additional information.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of FileInfo objects.
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

    def put(self, lpath: Path, rpath: Path, recursive: bool = False):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
            recursive:
                Whether to put recursively. lpath and rpath must be directories
        """

    def copy(self, src: Path, dest: Path, recursive: bool = False):
        """Copy the data from the source to the destination. This is an "in-bucket" copy.

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

        Returns:
            True if the path exists, False otherwise.
        """

    def generate_presigned_url(self, path: Path, expiration: int = 600) -> str:
        """Generate a signed URL for the path.

        Args:
            path:
                The path to the file.
            expiration:
                The expiration time in seconds.

        Returns:
            The signed URL.
        """

    def delete_bucket(self) -> None:
        """Delete the bucket."""

class PyGCSFSStorageClient:
    def __init__(self, settings: StorageSettings):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the s3 bucket.
        """

    def find(self, path: Optional[Path] = None) -> List[str]:
        """Returns all the files in the path.

        Args:
            path:
                The path to search for files.
        """

    def find_info(self, path: Optional[Path] = None) -> List[FileInfo]:
        """Returns all the files in the path with additional information.

        Args:
            path:
                The path to search for files.

        Returns:
            A list of FileInfo objects.
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

    def put(self, lpath: Path, rpath: Path, recursive: bool = False):
        """Put the data in the path.

        Args:
            lpath:
                The path to the local file.
            rpath:
                The path to the remote file.
            recursive:
                Whether to put recursively. lpath and rpath must be directories
        """

    def copy(self, src: Path, dest: Path, recursive: bool = False):
        """Copy the data from the source to the destination. This is an "in-bucket" copy.

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

        Returns:
            True if the path exists, False otherwise.
        """

    def generate_presigned_url(self, path: Path, expiration: int = 600) -> str:
        """Generate a signed URL for the path.

        Args:
            path:
                The path to the file.
            expiration:
                The expiration time in seconds.

        Returns:
            The signed URL.
        """

    def delete_bucket(self) -> None:
        """Delete the bucket."""
