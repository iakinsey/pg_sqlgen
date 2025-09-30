use async_trait::async_trait;

use crate::types::errors::ModelDriverError;

#[async_trait]
pub trait TextEncoderDriver: Send + Sync {
    async fn dimensions(&self) -> Result<usize, ModelDriverError>;
    async fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError>;
    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, ModelDriverError>;
}
