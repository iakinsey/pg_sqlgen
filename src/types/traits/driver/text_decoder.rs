pub trait TextDecoderDriver {
    fn decode(&self, input: &str) -> Result<String, ModelDriverError>;
}