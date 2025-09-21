use hf_hub::api::sync::ApiError;
use thiserror::Error;

use crate::types::errors::UtilError;

#[derive(Error, Debug)]
pub enum ModelDriverError {
    #[error("{0}")]
    Any(String),
    #[error("{0}")]
    DeviceError(String),
    #[error("unknown device: {0}")]
    UnknownDevice(String),
    #[error("model encoding error: {0}")]
    EncodeError(String),
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error(transparent)]
    TokenizerError(#[from] tokenizers::Error),
    #[error(transparent)]
    CandleCoreError(#[from] candle_core::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    UtilError(#[from] UtilError),
    #[error(transparent)]
    DTypeParseError(#[from] candle_core::DTypeParseError)
}
