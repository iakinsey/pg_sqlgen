use crate::{
    drivers::{LocalBertDriver, OllamaDriver},
    types::{
        errors::ModelDriverError,
        structs::model_profile::{ModelConfig, ModelProfile},
        traits::driver::{ModelDriver, TextEncoderDriver},
    },
};

pub struct ModelStore {}

impl ModelStore {
    pub fn create() {
        unimplemented!()
    }

    pub fn delete() {
        unimplemented!()
    }

    pub fn get_model_descriptions() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            (
                LocalBertDriver::ID,
                LocalBertDriver::NAME,
                LocalBertDriver::DESCRIPTION,
            ),
            (
                OllamaDriver::ID,
                OllamaDriver::NAME,
                OllamaDriver::DESCRIPTION,
            ),
        ]
    }

    pub fn get_model_profile(model_name: String) -> Result<ModelProfile, ModelDriverError> {
        unimplemented!()
    }

    pub fn get_text_encoder_model(
        model_name: String,
    ) -> Result<Box<dyn TextEncoderDriver>, ModelDriverError> {
        unimplemented!()
    }

    pub fn get_text_instruct_model() {
        unimplemented!()
    }
}
