use candle_core::{
    utils::{cuda_is_available, metal_is_available},
    Device,
};
use serde::{Deserialize, Serialize};

use crate::types::{errors::ModelDriverError, structs::ModelProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalBertProfile {
    pub model_name: String,
    pub revision: String,
    #[serde(default = "default_config_filename")]
    pub config_filename: String,
    #[serde(default = "default_tokenizer_filename")]
    pub tokenizer_filename: String,
    #[serde(default = "default_weights_filename")]
    pub weights_filename: String,
    #[serde(default = "default_compute_device")]
    pub compute_device: String,
}

fn default_config_filename() -> String {
    "config.json".to_string()
}

fn default_tokenizer_filename() -> String {
    "tokenizer.json".to_string()
}

fn default_weights_filename() -> String {
    "model.safetensors".to_string()
}

fn default_compute_device() -> String {
    if cuda_is_available() {
        return "cuda".to_string();
    } else if metal_is_available() {
        return "metal".to_string();
    }

    "cpu".to_string()
}

impl LocalBertProfile {
    pub fn from_model_profile(profile: ModelProfile) -> Result<Self, ModelDriverError> {
        unimplemented!();
    }

    pub fn get_device(&self) -> Result<Device, ModelDriverError> {
        match self.compute_device.as_str() {
            "cpu" => Ok(Device::Cpu),
            "cuda" => Ok(
                Device::new_cuda(0).map_err(|e| ModelDriverError::DeviceError(e.to_string()))?
            ),
            "metal" => {
                Ok(Device::new_metal(0)
                    .map_err(|e| ModelDriverError::DeviceError(e.to_string()))?)
            }
            _ => Err(ModelDriverError::UnknownDevice(
                self.compute_device.to_string(),
            )),
        }
    }
}
