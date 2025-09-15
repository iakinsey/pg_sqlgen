use crate::types::errors::ModelDriverError;

pub trait TextEncoderDriver {
    fn dimensions(&self) -> Result<usize, ModelDriverError>;
    fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError>;
    fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, ModelDriverError>;
}
