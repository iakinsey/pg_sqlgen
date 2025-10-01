use serde::{Deserialize, Serialize};

use crate::{
    drivers::{LocalBertDriver, OllamaDriver},
    types::{
        errors::ModelDriverError,
        structs::profiles::{LocalBertConfig, OllamaConfig},
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
};

#[derive(Serialize, Deserialize)]
pub struct ModelProfile {
    pub name: String,
    pub driver_name: String,
    pub config: ModelConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum ModelConfig {
    LocalBert(LocalBertConfig),
    Ollama(OllamaConfig),
}

impl ModelProfile {
    pub fn get_text_encoder_model(&self) -> Result<Box<dyn TextEncoderDriver>, ModelDriverError> {
        let driver: Box<dyn TextEncoderDriver> = match &self.config {
            ModelConfig::LocalBert(cfg) => Box::new(LocalBertDriver::new(&cfg)?),
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
        };

        Ok(driver)
    }

    pub fn get_text_instruct_model(&self) -> Result<Box<dyn TextInstructDriver>, ModelDriverError> {
        let driver: Box<dyn TextInstructDriver> = match &self.config {
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
            _ => {
                return Err(ModelDriverError::UnsupportedModelConfig(
                    self.driver_name.clone(),
                ))
            }
        };

        Ok(driver)
    }
}
