use thiserror::Error;

use crate::types::errors::ModelDriverError;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("{0}")]
    Any(String),
    #[error(transparent)]
    ModelDriverError(#[from] ModelDriverError),
}
