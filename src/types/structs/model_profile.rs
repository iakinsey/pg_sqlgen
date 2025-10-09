use pgrx::spi::SpiTupleTable;
use serde::{Deserialize, Serialize};

use crate::{
    drivers::{LocalBertDriver, OllamaDriver},
    types::{
        errors::ModelDriverError,
        structs::profiles::{LocalBertConfig, OllamaConfig},
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
    utils::sql::get_column,
};

pub struct ModelProfile {
    pub name: String,
    pub config: ModelConfig,
    pub generate_prompt: String,
    pub filter_prompt: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum ModelConfig {
    LocalBert(LocalBertConfig),
    Ollama(OllamaConfig),
}

impl ModelProfile {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, ModelDriverError> {
        let name: String = get_column(&row, "name")?;
        let config_json: String = get_column(&row, "config")?;
        let config: ModelConfig = serde_json::from_str(&config_json)?;
        let generate_prompt: String = get_column(&row, "generate_prompt")?;
        let filter_prompt: String = get_column(&row, "filter_prompt")?;

        Ok(Self {
            name,
            config,
            generate_prompt,
            filter_prompt,
        })
    }

    pub fn has_valid_config(&self) -> bool {
        match &self.config {
            ModelConfig::LocalBert(_) => true,
            ModelConfig::Ollama(_) => true,
        }
    }

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
            _ => return Err(ModelDriverError::UnsupportedModelConfig(self.name.clone())),
        };

        Ok(driver)
    }
}
