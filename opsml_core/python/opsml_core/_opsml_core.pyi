from pathlib import Path
from typing import List, Optional
from enum import Enum

class StorageType(str, Enum):
    Google = "google"
    AWS = "aws"
    Local = "local"

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

    def __str__(self) -> str:
        """Return a string representation of the FileInfo object."""

class ApiSettings:
    @property
    def base_url(self) -> str:
        """The base URL of the API."""

    @property
    def use_auth(self) -> bool:
        """Whether to use authentication."""

    @property
    def opsml_dir(self) -> str:
        """The directory of the OPSML file."""

    @property
    def scouter_dir(self) -> str:
        """The directory of the Scouter file."""

    @property
    def username(self) -> str:
        """The username."""

    @property
    def password(self) -> str:
        """The password."""

    @property
    def auth_token(self) -> str:
        """The authentication token."""

    @property
    def prod_token(self) -> str:
        """The production token."""

class OpsmlStorageSettings:
    @property
    def storage_uri(self) -> str:
        """The storage URI."""

    @property
    def client_mode(self) -> bool:
        """Whether to use the client."""

    @property
    def api_settings(self) -> ApiSettings:
        """The API settings."""

    @property
    def storage_type(self) -> StorageType:
        """The storage type."""

class OpsmlConfig:
    def __init__(self, client_mode: Optional[bool] = None) -> None:
        """Initialize the OpsmlConfig.

        Args:
            client_mode:
                Whether to use the client. By default, OpsML will determine whether
                to run in client mode based on the provided OPSML_TRACKING_URI. This attribute
                will override that behavior. Default is None.
        """

    @property
    def app_name(self) -> str:
        """The name of the application."""

    @property
    def app_env(self) -> str:
        """The environment of the application."""

    @property
    def app_version(self) -> str:
        """The version of the application."""

    @property
    def opsml_storage_uri(self) -> str:
        """The storage URI for Opsml."""

    @property
    def opsml_tracking_uri(self) -> str:
        """The tracking URI for Opsml."""

    @property
    def opsml_prod_token(self) -> str:
        """The production token for Opsml."""

    @property
    def opsml_proxy_root(self) -> str:
        """The proxy root for Opsml."""

    @property
    def opsml_registry_path(self) -> str:
        """The registry path for Opsml."""

    @property
    def opsml_testing(self) -> bool:
        """Indicates if Opsml is in testing mode."""

    @property
    def download_chunk_size(self) -> int:
        """The download chunk size."""

    @property
    def upload_chunk_size(self) -> int:
        """The upload chunk size."""

    @property
    def opsml_jwt_secret(self) -> str:
        """The JWT secret for Opsml."""

    @property
    def opsml_jwt_algorithm(self) -> str:
        """The JWT algorithm for Opsml."""

    @property
    def opsml_username(self) -> Optional[str]:
        """The username for Opsml."""

    @property
    def opsml_password(self) -> Optional[str]:
        """The password for Opsml."""

    @property
    def scouter_server_uri(self) -> Optional[str]:
        """The server URI for Scouter."""

    @property
    def scouter_username(self) -> Optional[str]:
        """The username for Scouter."""

    @property
    def scouter_password(self) -> Optional[str]:
        """The password for Scouter."""

    @property
    def scouter_auth(self) -> bool:
        """Indicates if Scouter authentication is enabled."""

    @property
    def opsml_auth(self) -> bool:
        """Indicates if Opsml authentication is enabled."""

    def storage_settings(self) -> OpsmlStorageSettings:
        """Get the storage settings."""

class PyFileSystemStorage:
    def __init__(self, settings: OpsmlStorageSettings):
        """Initialize the storage client.

        Args:
            bucket_name:
                The name of the s3 bucket.
        """

    def name(self) -> str:
        """The name of the storage client."""

    def storage_type(self) -> StorageType:
        """The storage type."""

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
