from pathlib import Path
from typing import List, Optional, Any, Dict
from enum import Enum

class OpsmlError(Exception):
    def __init__(self, message: str) -> None: ...
    def __str__(self) -> str: ...

class HuggingFaceORTModel:
    OrtAudioClassification: "HuggingFaceORTModel"
    OrtAudioFrameClassification: "HuggingFaceORTModel"
    OrtAudioXVector: "HuggingFaceORTModel"
    OrtCustomTasks: "HuggingFaceORTModel"
    OrtCtc: "HuggingFaceORTModel"
    OrtFeatureExtraction: "HuggingFaceORTModel"
    OrtImageClassification: "HuggingFaceORTModel"
    OrtMaskedLm: "HuggingFaceORTModel"
    OrtMultipleChoice: "HuggingFaceORTModel"
    OrtQuestionAnswering: "HuggingFaceORTModel"
    OrtSemanticSegmentation: "HuggingFaceORTModel"
    OrtSequenceClassification: "HuggingFaceORTModel"
    OrtTokenClassification: "HuggingFaceORTModel"
    OrtSeq2SeqLm: "HuggingFaceORTModel"
    OrtSpeechSeq2Seq: "HuggingFaceORTModel"
    OrtVision2Seq: "HuggingFaceORTModel"
    OrtPix2Struct: "HuggingFaceORTModel"
    OrtCausalLm: "HuggingFaceORTModel"
    OrtOptimizer: "HuggingFaceORTModel"
    OrtQuantizer: "HuggingFaceORTModel"
    OrtTrainer: "HuggingFaceORTModel"
    OrtSeq2SeqTrainer: "HuggingFaceORTModel"
    OrtTrainingArguments: "HuggingFaceORTModel"
    OrtSeq2SeqTrainingArguments: "HuggingFaceORTModel"
    OrtStableDiffusionPipeline: "HuggingFaceORTModel"
    OrtStableDiffusionImg2ImgPipeline: "HuggingFaceORTModel"
    OrtStableDiffusionInpaintPipeline: "HuggingFaceORTModel"
    OrtStableDiffusionXlPipeline: "HuggingFaceORTModel"
    OrtStableDiffusionXlImg2ImgPipeline: "HuggingFaceORTModel"

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

class HuggingFaceOnnxArgs:
    ort_type: HuggingFaceORTModel
    provider: str
    quantize: bool
    config: Optional[Any]

    def __init__(
        self,
        ort_type: HuggingFaceORTModel,
        provider: str,
        quantize: bool = False,
        config: Optional[Any] = None,
    ) -> None:
        """Optional Args to use with a huggingface model

        Args:
            ort_type:
                Optimum onnx class name
            provider:
                Onnx runtime provider to use
            config:
                Optional optimum config to use
        """

class TorchOnnxArgs:
    input_names: list[str]
    output_names: list[str]
    dynamic_axes: Optional[Dict[str, Dict[int, str]]]
    do_constant_folding: bool
    export_params: bool
    verbose: bool

    def __init__(
        self,
        input_names: list[str],
        output_names: list[str],
        dynamic_axes: Optional[Dict[str, Dict[int, str]]] = None,
        do_constant_folding: bool = True,
        export_params: bool = True,
        verbose: bool = True,
    ) -> None:
        """Optional arguments to pass to torch when converting to onnx

        Args:
            input_names:
                Optional list containing input names for model inputs.
            output_names:
                Optional list containing output names for model outputs.
            dynamic_axes:
                Optional PyTorch attribute that defines dynamic axes
            constant_folding:
                Whether to use constant folding optimization. Default is True
        """
    def model_dump(self) -> dict[str, Any]:
        """Dump onnx args to dictionary

        Returns:
            Dictionary containing model information
        """

class TorchSaveArgs:
    as_state_dict: bool

    def __init__(self, as_state_dict: bool = False) -> None:
        """Optional arguments to pass to torch when saving a model

        Args:
            as_state_dict:
                Whether to save the model as a state dict. Default is False
        """
