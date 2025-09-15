use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("{0}")]
    Any(String),
    #[error(transparent)]
    CandleCoreError(#[from] candle_core::Error),
}
