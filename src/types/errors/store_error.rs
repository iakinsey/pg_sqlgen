use thiserror::Error;

use crate::types::errors::{ModelDriverError, UtilError};

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("{0}")]
    Any(String),
    #[error(transparent)]
    ModelDriverError(#[from] ModelDriverError),
    #[error("model doesn't exist: {0}")]
    ModelDoesntExist(String),
    #[error("config value doesn't exist: {0}")]
    ConfigDoesntExist(String),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    SpiError(#[from] pgrx::spi::Error),
    #[error(transparent)]
    UtilError(#[from] UtilError),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
}
