use hf_hub::api::sync::ApiError;
use std::sync::PoisonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlgenError {
    #[error("{0}")]
    Any(String),
    #[error("{0}")]
    ParseError(String),
    #[error("{0}")]
    ResponseError(String),
    #[error("{0}")]
    DeviceError(String),
    #[error("unsupported model config: {0}")]
    UnsupportedModelConfig(String),
    #[error("unknown device: {0}")]
    UnknownDevice(String),
    #[error("model encoding error: {0}")]
    EncodeError(String),
    #[error("model doesn't exist: {0}")]
    ModelDoesntExist(String),
    #[error("failed to get encoding: {0} {1} {2}")]
    EncodingError(String, String, String),
    #[error("table doesn't exist: {0}")]
    TableDoesntExist(String),
    #[error("config value doesn't exist: {0}")]
    ConfigDoesntExist(String),
    #[error("mutex poisoned")]
    Poisoned,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("no schema available")]
    NoSchema(),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    SpiError(#[from] pgrx::spi::Error),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error(transparent)]
    TokenizerError(#[from] tokenizers::Error),
    #[error(transparent)]
    CandleCoreError(#[from] candle_core::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DTypeParseError(#[from] candle_core::DTypeParseError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("{0}")]
    ColumnParseFailed(String),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl<T> From<PoisonError<T>> for SqlgenError {
    fn from(_: PoisonError<T>) -> Self {
        SqlgenError::Poisoned
    }
}
