// Mistral 7b

use crate::types::{
    errors::ModelDriverError,
    structs::{profiles::LocalMistralConfig, ModelProfile},
    traits::driver::{ModelDriver, TextDecoderDriver},
};

pub struct LocalMistralDriver {}

impl ModelDriver for LocalMistralDriver {
    const NAME: &'static str = "Local Mistral";
    const DESCRIPTION: &'static str = "A fast, open-weight large language model, runs locally.";
}

impl LocalMistralDriver {
    pub fn new(mistral_config: LocalMistralConfig) -> Result<Self, ModelDriverError> {
        unimplemented!()
    }
}

impl TextDecoderDriver for LocalMistralDriver {
    fn decode(&self, input: &str) -> Result<String, ModelDriverError> {
        unimplemented!()
    }

    fn decode_many(&self, inputs: &[&str]) -> Result<Vec<String>, ModelDriverError> {
        unimplemented!()
    }
}
