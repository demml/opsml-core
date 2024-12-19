from opsml_core import (
    HuggingFaceORTModel,
    HuggingFaceOnnxArgs,
    OpsmlError,
    TorchOnnxArgs,
    Feature,
    OnnxSchema,
    DataSchema,
    Description,
    ModelInterfaceArgs,
    SklearnModelInterfaceArgs,
    CatBoostModelInterfaceArgs,
    HuggingFaceModelInterfaceArgs,
    HuggingFaceOnnxSaveArgs,
    LightGBMModelInterfaceArgs,
    LightningInterfaceArgs,
    TorchInterfaceArgs,
    TensorFlowInterfaceArgs,
    VowpalWabbitInterfaceArgs,
    XGBoostModelInterfaceArgs,
    ModelInterfaceArgsEnum,
    CommonKwargs,
)


def create_common_args():
    feature_map = {"feature1": Feature("type1", [1, 2, 3], {"arg1": "value1"})}
    metadata = {"key1": "value1"}
    return feature_map, metadata


def test_model_interface_args_creation():
    feature_map = {"feature1": Feature("type1", [1, 2, 3], {"arg1": "value1"})}
    metadata = {"key1": "value1"}

    args = ModelInterfaceArgs(
        task_type="classification",
        model_type="sklearn",
        data_type="tabular",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="csv",
        metadata=metadata,
    )

    assert args.task_type == "classification"
    assert args.model_type == "sklearn"
    assert args.data_type == "tabular"
    assert args.modelcard_uid == "1234"
    assert args.feature_map == feature_map
    assert args.sample_data_interface_type == "csv"
    assert args.metadata == metadata


def test_sklearn_model_interface_args_creation():
    feature_map = {"feature1": Feature("type1", [1, 2, 3], {"arg1": "value1"})}
    metadata = {"key1": "value1"}

    args = SklearnModelInterfaceArgs(
        task_type="classification",
        model_type="sklearn",
        data_type="tabular",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="csv",
        preprocessor_name="StandardScaler",
        metadata=metadata,
    )

    assert args.task_type == "classification"
    assert args.model_type == "sklearn"
    assert args.data_type == "tabular"
    assert args.modelcard_uid == "1234"
    assert args.feature_map == feature_map
    assert args.sample_data_interface_type == "csv"
    assert args.preprocessor_name == "StandardScaler"
    assert args.metadata == metadata


def test_model_interface_args_enum_creation():
    feature_map = {"feature1": Feature("type1", [1, 2, 3], {"arg1": "value1"})}
    metadata = {"key1": "value1"}
    onnx_args = HuggingFaceOnnxSaveArgs(
        HuggingFaceORTModel.OrtAudioClassification,
        "provider",
        True,
    )
    backend = CommonKwargs.Pytorch

    args = HuggingFaceModelInterfaceArgs(
        task_type="classification",
        model_type="huggingface",
        data_type="text",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="json",
        preprocessor_name="BertTokenizer",
        is_pipeline=True,
        backend=backend,
        onnx_args=onnx_args,
        tokenizer_name="bert-base-uncased",
        feature_extractor_name="bert-feature-extractor",
        metadata=metadata,
    )

    enum_args = ModelInterfaceArgsEnum(args)

    assert isinstance(enum_args, ModelInterfaceArgsEnum)
    assert enum_args.type_name() == "HuggingFace"


def test_model_interface_args_enum_lightgbm():
    feature_map, metadata = create_common_args()

    args = LightGBMModelInterfaceArgs(
        task_type="classification",
        model_type="lightgbm",
        data_type="tabular",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="csv",
        preprocessor_name="StandardScaler",
        metadata=metadata,
    )

    enum_args = ModelInterfaceArgsEnum(args)
    assert isinstance(enum_args, ModelInterfaceArgsEnum)
    assert enum_args.type_name() == "LightGBM"


def test_model_interface_args_enum_lightning():
    feature_map, metadata = create_common_args()
    onnx_args = TorchOnnxArgs(
        input_names=["input"],
        output_names=["output"],
    )

    args = LightningInterfaceArgs(
        task_type="classification",
        model_type="lightning",
        data_type="tabular",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="csv",
        preprocessor_name="StandardScaler",
        onnx_args=onnx_args,
        metadata=metadata,
    )

    enum_args = ModelInterfaceArgsEnum(args)
    assert isinstance(enum_args, ModelInterfaceArgsEnum)
    assert enum_args.type_name() == "Lightning"


def test_model_interface_args_enum_sklearn():
    feature_map, metadata = create_common_args()

    args = SklearnModelInterfaceArgs(
        task_type="classification",
        model_type="sklearn",
        data_type="tabular",
        modelcard_uid="1234",
        feature_map=feature_map,
        sample_data_interface_type="csv",
        preprocessor_name="StandardScaler",
        metadata=metadata,
    )

    enum_args = ModelInterfaceArgsEnum(args)
    assert isinstance(enum_args, ModelInterfaceArgsEnum)
    assert enum_args.type_name() == "Sklearn"
