use crate::types::errors::ModelDriverError;

pub trait TextDecoderDriver {
    fn decode(&self, input: &str) -> Result<String, ModelDriverError>;
    fn decode_many(&self, inputs: &[&str]) -> Result<Vec<String>, ModelDriverError>;
}
