use crate::cards::model::{HuggingFaceORTModel, TorchOnnxArgs, TorchSaveArgs};
use crate::shared::CommonKwargs;
use crate::Feature;
use opsml_error::error::OpsmlError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl ModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, metadata=None))]
    fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        ModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SklearnModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl SklearnModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, metadata=None))]
    fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        SklearnModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatBoostModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl CatBoostModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, metadata=None))]
    fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        CatBoostModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HuggingFaceOnnxSaveArgs {
    ort_type: HuggingFaceORTModel,
    provider: String,
    quantize: bool,
}
#[pymethods]
impl HuggingFaceOnnxSaveArgs {
    #[new]
    pub fn new(ort_type: HuggingFaceORTModel, provider: String, quantize: bool) -> Self {
        HuggingFaceOnnxSaveArgs {
            ort_type,
            provider,
            quantize,
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HuggingFaceModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub is_pipeline: bool,
    #[pyo3(get)]
    pub backend: CommonKwargs,
    #[pyo3(get)]
    pub onnx_args: HuggingFaceOnnxSaveArgs,
    #[pyo3(get)]
    pub tokenizer_name: String,
    #[pyo3(get)]
    pub feature_extractor_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl HuggingFaceModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, is_pipeline, backend, onnx_args, tokenizer_name, feature_extractor_name, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        is_pipeline: bool,
        backend: CommonKwargs,
        onnx_args: HuggingFaceOnnxSaveArgs,
        tokenizer_name: String,
        feature_extractor_name: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        HuggingFaceModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            is_pipeline,
            backend,
            onnx_args,
            tokenizer_name,
            feature_extractor_name,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightGBMModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl LightGBMModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        LightGBMModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub onnx_args: Option<TorchOnnxArgs>,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl LightningInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, onnx_args=None, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        onnx_args: Option<TorchOnnxArgs>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        LightningInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            onnx_args,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorchInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub onnx_args: Option<TorchOnnxArgs>,
    #[pyo3(get)]
    pub save_args: TorchSaveArgs,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl TorchInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, onnx_args=None, save_args=None, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        onnx_args: Option<TorchOnnxArgs>,
        save_args: Option<TorchSaveArgs>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        TorchInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            save_args: save_args.unwrap_or(TorchSaveArgs::new(None)),
            onnx_args,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TensorFlowInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl TensorFlowInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, preprocessor_name, sample_data_interface_type, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        preprocessor_name: String,
        sample_data_interface_type: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        TensorFlowInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            preprocessor_name,
            sample_data_interface_type,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VowpalWabbitInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub arguments: String,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl VowpalWabbitInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, arguments, sample_data_interface_type, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        arguments: String,
        sample_data_interface_type: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        VowpalWabbitInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            arguments,
            sample_data_interface_type,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XGBoostModelInterfaceArgs {
    #[pyo3(get)]
    pub task_type: String,
    #[pyo3(get)]
    pub model_type: String,
    #[pyo3(get)]
    pub data_type: String,
    #[pyo3(get)]
    pub modelcard_uid: String,
    #[pyo3(get)]
    pub feature_map: HashMap<String, Feature>,
    #[pyo3(get)]
    pub sample_data_interface_type: String,
    #[pyo3(get)]
    pub preprocessor_name: String,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl XGBoostModelInterfaceArgs {
    #[new]
    #[pyo3(signature = (task_type, model_type, data_type, modelcard_uid, feature_map, sample_data_interface_type, preprocessor_name, metadata=None))]
    pub fn new(
        task_type: String,
        model_type: String,
        data_type: String,
        modelcard_uid: String,
        feature_map: HashMap<String, Feature>,
        sample_data_interface_type: String,
        preprocessor_name: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        XGBoostModelInterfaceArgs {
            task_type,
            model_type,
            data_type,
            modelcard_uid,
            feature_map,
            sample_data_interface_type,
            preprocessor_name,
            metadata: metadata.unwrap_or_default(),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModelInterfaceArgsEnum {
    HuggingFace(HuggingFaceModelInterfaceArgs),
    LightGBM(LightGBMModelInterfaceArgs),
    Lightning(LightningInterfaceArgs),
    Sklearn(SklearnModelInterfaceArgs),
    TensorFlow(TensorFlowInterfaceArgs),
    Torch(TorchInterfaceArgs),
    Vowpal(VowpalWabbitInterfaceArgs),
    XGBoost(XGBoostModelInterfaceArgs),
    CatBoost(CatBoostModelInterfaceArgs),
    Base(ModelInterfaceArgs),
}

#[pymethods]
impl ModelInterfaceArgsEnum {
    #[new]
    #[pyo3(signature = (interface_args))]
    pub fn new(interface_args: &Bound<'_, PyAny>) -> PyResult<Self> {
        if interface_args.is_instance_of::<HuggingFaceModelInterfaceArgs>() {
            let args: HuggingFaceModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract HuggingFaceModelInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::HuggingFace(args))
        } else if interface_args.is_instance_of::<LightGBMModelInterfaceArgs>() {
            let args: LightGBMModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract LightGBMModelInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::LightGBM(args))
        } else if interface_args.is_instance_of::<LightningInterfaceArgs>() {
            let args: LightningInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!("Failed to extract LightningInterfaceArgs: {}", e))
            })?;
            Ok(ModelInterfaceArgsEnum::Lightning(args))
        } else if interface_args.is_instance_of::<SklearnModelInterfaceArgs>() {
            let args: SklearnModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract SklearnModelInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::Sklearn(args))
        } else if interface_args.is_instance_of::<TensorFlowInterfaceArgs>() {
            let args: TensorFlowInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!("Failed to extract TensorFlowInterfaceArgs: {}", e))
            })?;
            Ok(ModelInterfaceArgsEnum::TensorFlow(args))
        } else if interface_args.is_instance_of::<TorchInterfaceArgs>() {
            let args: TorchInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!("Failed to extract TorchInterfaceArgs: {}", e))
            })?;
            Ok(ModelInterfaceArgsEnum::Torch(args))
        } else if interface_args.is_instance_of::<VowpalWabbitInterfaceArgs>() {
            let args: VowpalWabbitInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract VowpalWabbitInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::Vowpal(args))
        } else if interface_args.is_instance_of::<XGBoostModelInterfaceArgs>() {
            let args: XGBoostModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract XGBoostModelInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::XGBoost(args))
        } else if interface_args.is_instance_of::<CatBoostModelInterfaceArgs>() {
            let args: CatBoostModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!(
                    "Failed to extract CatBoostModelInterfaceArgs: {}",
                    e
                ))
            })?;
            Ok(ModelInterfaceArgsEnum::CatBoost(args))
        } else if interface_args.is_instance_of::<ModelInterfaceArgs>() {
            let args: ModelInterfaceArgs = interface_args.extract().map_err(|e| {
                OpsmlError::new_err(format!("Failed to extract ModelInterfaceArgs: {}", e))
            })?;
            Ok(ModelInterfaceArgsEnum::Base(args))
        } else {
            Err(OpsmlError::new_err("Invalid ModelInterfaceArgs type"))
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            ModelInterfaceArgsEnum::HuggingFace(_) => "HuggingFace",
            ModelInterfaceArgsEnum::LightGBM(_) => "LightGBM",
            ModelInterfaceArgsEnum::Lightning(_) => "Lightning",
            ModelInterfaceArgsEnum::Sklearn(_) => "Sklearn",
            ModelInterfaceArgsEnum::TensorFlow(_) => "TensorFlow",
            ModelInterfaceArgsEnum::Torch(_) => "Torch",
            ModelInterfaceArgsEnum::Vowpal(_) => "VowpalWabbit",
            ModelInterfaceArgsEnum::XGBoost(_) => "XGBoost",
            ModelInterfaceArgsEnum::CatBoost(_) => "CatBoost",
            ModelInterfaceArgsEnum::Base(_) => "Base",
        }
    }
}
