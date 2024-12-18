use crate::Feature;
use pyo3::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
struct HuggingFaceModelInterfaceArgs {
    task_type: String,
    model_type: String,
    data_type: String,
    modelcard_uid: String,
    feature_map: HashMap<String, Feature>,
    sample_data_interface: String,
    is_pipeline: bool,
    backend: String,
    tokenizer_name: String,
    feature_extractor_name: String,
}

#[pyclass]
#[derive(Debug)]
struct HuggingFaceOnnxArgs {
    pub ort_type: String,
    pub provider: String,
    pub quantize: bool,
    pub config: Option<PyObject>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HuggingFaceOnnxSaveArgs {
    ort_type: String,
    provider: String,
    quantize: bool,
}

// impl Serialize for HuggingFaceOnnxArgs
impl Serialize for HuggingFaceOnnxArgs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let save_args = HuggingFaceOnnxSaveArgs {
            ort_type: self.ort_type.clone(),
            provider: self.provider.clone(),
            quantize: self.quantize,
        };
        save_args.serialize(serializer)
    }
}

// impl Deserialize for HuggingFaceOnnxArgs
impl<'de> Deserialize<'de> for HuggingFaceOnnxArgs {
    fn deserialize<D>(deserializer: D) -> Result<HuggingFaceOnnxArgs, D::Error>
    where
        D: Deserializer<'de>,
    {
        let save_args = HuggingFaceOnnxSaveArgs::deserialize(deserializer)?;
        Ok(HuggingFaceOnnxArgs {
            ort_type: save_args.ort_type,
            provider: save_args.provider,
            quantize: save_args.quantize,
            config: None,
        })
    }
}
