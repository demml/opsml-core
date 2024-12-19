from opsml_core import (
    HuggingFaceORTModel,
    HuggingFaceOnnxArgs,
    OpsmlError,
    TorchOnnxArgs,
)
from optimum.onnxruntime.configuration import AutoQuantizationConfig  # type: ignore
import pytest


def test_hugging_face_ort_model():
    args = HuggingFaceOnnxArgs(
        ort_type=HuggingFaceORTModel.OrtAudioClassification,
        provider="CPUExecutionProvider",
    )

    assert args.quantize is False

    args = HuggingFaceOnnxArgs(
        ort_type=HuggingFaceORTModel.OrtAudioClassification,
        provider="CPUExecutionProvider",
        quantize=True,
    )

    assert args.quantize is True

    with pytest.raises(OpsmlError) as error:
        args = HuggingFaceOnnxArgs(
            ort_type=HuggingFaceORTModel.OrtAudioClassification,
            provider="CPUExecutionProvider",
            quantize=True,
            config="fail",
        )

    assert (
        str(error.value)
        == "config must be an instance of AutoQuantizationConfig, ORTConfig, or QuantizationConfig"
    )

    args = HuggingFaceOnnxArgs(
        ort_type=HuggingFaceORTModel.OrtAudioClassification,
        provider="CPUExecutionProvider",
        quantize=True,
        config=AutoQuantizationConfig.avx512_vnni(is_static=False, per_channel=False),
    )


def test_torch_onnx_args():
    args = TorchOnnxArgs(
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={"input": {0: "batch"}},
        do_constant_folding=True,
        export_params=True,
        verbose=True,
    )

    assert args.do_constant_folding is True
    assert args.export_params is True

    # convert to dictionary
    args_dict = args.model_dump()

    assert args_dict == {
        "input_names": ["input"],
        "output_names": ["output"],
        "dynamic_axes": {"input": {0: "batch"}},
        "do_constant_folding": True,
        "export_params": True,
        "verbose": True,
    }
