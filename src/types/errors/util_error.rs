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
}
