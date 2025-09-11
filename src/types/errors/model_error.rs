use thiserror::error;

#[derive(Error, Debug)]
pub enum ModelDriverError {
    #[error("{0}")]
    Any(String),
}
