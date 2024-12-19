use opsml_error::error::OpsmlError;
use opsml_settings::config::OpsmlConfig;
use opsml_types::cards::model::{
    CatBoostModelInterfaceArgs, HuggingFaceModelInterfaceArgs, LightGBMModelInterfaceArgs,
    LightningInterfaceArgs, ModelInterfaceArgs, ModelInterfaceArgsEnum, SklearnModelInterfaceArgs,
    TensorFlowInterfaceArgs, TorchInterfaceArgs, VowpalWabbitInterfaceArgs,
    XGBoostModelInterfaceArgs,
};
use opsml_types::cards::{
    DataSchema, Description, Feature, HuggingFaceORTModel, HuggingFaceOnnxArgs,
    HuggingFaceOnnxSaveArgs, OnnxSchema, TorchOnnxArgs, TorchSaveArgs, VersionType,
};
use opsml_types::shared::{CommonKwargs, SaveName, Suffix};

use pyo3::prelude::*;

#[pymodule]
fn _opsml_core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    // errors
    _m.add("OpsmlError", _m.py().get_type::<OpsmlError>())?;

    // config
    _m.add_class::<OpsmlConfig>()?;

    // shared
    _m.add_class::<CommonKwargs>()?;
    _m.add_class::<SaveName>()?;
    _m.add_class::<Suffix>()?;

    // cards (types that are used across cards)
    _m.add_class::<HuggingFaceOnnxArgs>()?;
    _m.add_class::<HuggingFaceORTModel>()?;
    _m.add_class::<HuggingFaceOnnxSaveArgs>()?;
    _m.add_class::<TorchOnnxArgs>()?;
    _m.add_class::<TorchSaveArgs>()?;
    _m.add_class::<Feature>()?;
    _m.add_class::<Description>()?;
    _m.add_class::<VersionType>()?;
    _m.add_class::<DataSchema>()?;
    _m.add_class::<OnnxSchema>()?;

    // Model Interface args
    _m.add_class::<ModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<CatBoostModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<HuggingFaceModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<LightGBMModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<LightningInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<SklearnModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<TensorFlowInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<TorchInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<VowpalWabbitInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<XGBoostModelInterfaceArgs>()?; // TODO: pyi
    _m.add_class::<ModelInterfaceArgsEnum>()?; // TODO: pyi

    Ok(())
}
