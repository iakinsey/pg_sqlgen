use async_trait::async_trait;

use crate::types::errors::SqlgenError;

// Interface for converting text into vectors.
#[async_trait]
pub trait TextEncoderDriver: Send + Sync {
    async fn dimensions(&self) -> Result<usize, SqlgenError>;
    async fn encode(&self, input: &str) -> Result<Vec<f32>, SqlgenError>;
    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, SqlgenError>;
}
