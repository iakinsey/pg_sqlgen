pub trait TextEncoderDriver {
    fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError>;
}
