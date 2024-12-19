from opsml_core import CommonKwargs, SaveName, Suffix
import pytest


@pytest.mark.parametrize(
    "variant, expected_string",
    [
        (CommonKwargs.IsPipeline, "is_pipeline"),
        (CommonKwargs.ModelType, "model_type"),
        (CommonKwargs.ModelClass, "model_class"),
        (CommonKwargs.ModelArch, "model_arch"),
        (CommonKwargs.PreprocessorName, "preprocessor_name"),
        (CommonKwargs.Preprocessor, "preprocessor"),
        (CommonKwargs.TaskType, "task_type"),
        (CommonKwargs.Model, "model"),
        (CommonKwargs.Undefined, "undefined"),
        (CommonKwargs.Backend, "backend"),
        (CommonKwargs.Pytorch, "pytorch"),
        (CommonKwargs.Tensorflow, "tensorflow"),
        (CommonKwargs.SampleData, "sample_data"),
        (CommonKwargs.Onnx, "onnx"),
        (CommonKwargs.LoadType, "load_type"),
        (CommonKwargs.DataType, "data_type"),
        (CommonKwargs.Tokenizer, "tokenizer"),
        (CommonKwargs.TokenizerName, "tokenizer_name"),
        (CommonKwargs.FeatureExtractor, "feature_extractor"),
        (CommonKwargs.FeatureExtractorName, "feature_extractor_name"),
        (CommonKwargs.Image, "image"),
        (CommonKwargs.Text, "text"),
        (CommonKwargs.VowpalArgs, "arguments"),
        (CommonKwargs.BaseVersion, "0.0.0"),
        (CommonKwargs.SampleDataInterfaceType, "sample_data_interface_type"),
    ],
)
def test_common_kwargs_as_string(variant, expected_string):
    assert variant.as_string() == expected_string
