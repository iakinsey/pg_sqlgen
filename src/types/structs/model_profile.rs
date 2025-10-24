use pgrx::spi::SpiTupleTable;
use serde::{Deserialize, Serialize};

use crate::{
    drivers::{LocalBertDriver, OllamaDriver, StubDriver},
    types::{
        errors::SqlgenError,
        structs::profiles::{LocalBertConfig, OllamaConfig, StubConfig},
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
    utils::sql::get_column,
};

pub struct ModelProfile {
    pub name: String,
    pub config: ModelConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "config")]
pub enum ModelConfig {
    LocalBert(LocalBertConfig),
    Ollama(OllamaConfig),
    Stub(StubConfig),
}

impl ModelProfile {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, SqlgenError> {
        let name: String = get_column(&row, "name")?;
        let config_json: String = get_column(&row, "config")?;
        let config: ModelConfig = serde_json::from_str(&config_json)?;

        Ok(Self { name, config })
    }

    pub fn has_valid_config(&self) -> bool {
        match &self.config {
            ModelConfig::LocalBert(_) => true,
            ModelConfig::Ollama(_) => true,
            ModelConfig::Stub(_) => true,
        }
    }

    pub fn get_text_encoder_model(&self) -> Result<Box<dyn TextEncoderDriver>, SqlgenError> {
        let driver: Box<dyn TextEncoderDriver> = match &self.config {
            ModelConfig::LocalBert(cfg) => Box::new(LocalBertDriver::new(&cfg)?),
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
            ModelConfig::Stub(cfg) => Box::new(StubDriver::new(cfg)?),
        };

        Ok(driver)
    }

    pub fn get_text_instruct_model(&self) -> Result<Box<dyn TextInstructDriver>, SqlgenError> {
        let driver: Box<dyn TextInstructDriver> = match &self.config {
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
            ModelConfig::Stub(cfg) => Box::new(StubDriver::new(cfg)?),
            _ => return Err(SqlgenError::UnsupportedModelConfig(self.name.clone())),
        };

        Ok(driver)
    }
}
