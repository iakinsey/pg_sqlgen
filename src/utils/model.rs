use candle_core::{
    utils::{cuda_is_available, metal_is_available},
    Device,
};

use crate::types::errors::ModelDriverError;

pub fn get_device(compute_device: &str) -> Result<Device, ModelDriverError> {
    match compute_device {
        "cpu" => Ok(Device::Cpu),
        "cuda" => {
            Ok(Device::new_cuda(0).map_err(|e| ModelDriverError::DeviceError(e.to_string()))?)
        }
        "metal" => {
            Ok(Device::new_metal(0).map_err(|e| ModelDriverError::DeviceError(e.to_string()))?)
        }
        _ => Err(ModelDriverError::UnknownDevice(compute_device.to_string())),
    }
}

pub fn default_compute_device() -> String {
    if cuda_is_available() {
        return "cuda".to_string();
    } else if metal_is_available() {
        return "metal".to_string();
    }

    "cpu".to_string()
}
