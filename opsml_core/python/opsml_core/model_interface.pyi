from typing import Dict, Optional, Any

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

class ModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class SklearnModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class CatBoostModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class HuggingFaceOnnxSaveArgs:
    ort_type: "HuggingFaceORTModel"
    provider: str
    quantize: bool

    def __init__(
        self, ort_type: "HuggingFaceORTModel", provider: str, quantize: bool
    ) -> None: ...

class HuggingFaceModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    is_pipeline: bool
    backend: "CommonKwargs"
    onnx_args: HuggingFaceOnnxSaveArgs
    tokenizer_name: str
    feature_extractor_name: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        is_pipeline: bool,
        backend: "CommonKwargs",
        onnx_args: HuggingFaceOnnxSaveArgs,
        tokenizer_name: str,
        feature_extractor_name: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class LightGBMModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class LightningInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    onnx_args: Optional["TorchOnnxArgs"]
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        onnx_args: Optional["TorchOnnxArgs"] = None,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class TorchInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    onnx_args: Optional["TorchOnnxArgs"]
    save_args: "TorchSaveArgs"
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        onnx_args: Optional["TorchOnnxArgs"] = None,
        save_args: Optional["TorchSaveArgs"] = None,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class TensorFlowInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    preprocessor_name: str
    sample_data_interface_type: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        preprocessor_name: str,
        sample_data_interface_type: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class VowpalWabbitInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    arguments: str
    sample_data_interface_type: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        arguments: str,
        sample_data_interface_type: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class XGBoostModelInterfaceArgs:
    task_type: str
    model_type: str
    data_type: str
    modelcard_uid: str
    feature_map: Dict[str, "Feature"]
    sample_data_interface_type: str
    preprocessor_name: str
    metadata: Dict[str, str]

    def __init__(
        self,
        task_type: str,
        model_type: str,
        data_type: str,
        modelcard_uid: str,
        feature_map: Dict[str, "Feature"],
        sample_data_interface_type: str,
        preprocessor_name: str,
        metadata: Optional[Dict[str, str]] = None,
    ) -> None: ...

class ModelInterfaceArgsEnum:
    Huggingface: "HuggingFaceModelInterfaceArgs"
    Lightgbm: "LightGBMModelInterfaceArgs"
    Lightning: "LightningInterfaceArgs"
    Sklearn: "SklearnModelInterfaceArgs"
    Tensorflow: "TensorFlowInterfaceArgs"
    Torch: "TorchInterfaceArgs"
    Vowpal: "VowpalWabbitInterfaceArgs"
    Xgboost: "XGBoostModelInterfaceArgs"
    CatBoost: "CatBoostModelInterfaceArgs"
    Base: "ModelInterfaceArgs"

    def __init__(self, interface_args: Any) -> None: ...
