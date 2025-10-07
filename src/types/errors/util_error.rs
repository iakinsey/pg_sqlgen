use std::sync::PoisonError;

use pgrx::spi;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("{0}")]
    Any(String),
    #[error("{0}")]
    ColumnParseFailed(String),
    #[error(transparent)]
    CandleCoreError(#[from] candle_core::Error),
    #[error(transparent)]
    SpiError(#[from] spi::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("mutex poisoned")]
    Poisoned,
    #[error("not found: {0}")]
    NotFound(String),
}

impl<T> From<PoisonError<T>> for UtilError {
    fn from(_: PoisonError<T>) -> Self {
        UtilError::Poisoned
    }
}
