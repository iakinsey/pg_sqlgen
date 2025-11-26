use pgrx::spi::SpiTupleTable;
use serde::{Deserialize, Serialize};

use crate::{
    drivers::{OllamaDriver, OpenAICompletionsDriver, OpenAIEmbeddingsDriver, StubDriver},
    types::{
        errors::SqlgenError,
        structs::profiles::{
            OllamaConfig, OpenAICompletionsConfig, OpenAIEmbeddingsConfig, StubConfig,
        },
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
    utils::sql::get_column,
};

// ModelProfile contains information necesasry for engines to create drivers.
// State management for this struct is handled by `ModelStore`.
pub struct ModelProfile {
    pub name: String,
    pub config: ModelConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "config")]
pub enum ModelConfig {
    OpenAICompletions(OpenAICompletionsConfig),
    OpenAIEmbeddings(OpenAIEmbeddingsConfig),
    Ollama(OllamaConfig),
    Stub(StubConfig),
}

impl ModelProfile {
    pub fn from_row(row: SpiTupleTable) -> Result<Self, SqlgenError> {
        let name: String = get_column(&row, "model_name")?;
        let config_json: String = get_column(&row, "config")?;
        let config: ModelConfig = serde_json::from_str(&config_json)?;

        Ok(Self { name, config })
    }

    pub fn get_text_encoder_model(&self) -> Result<Box<dyn TextEncoderDriver>, SqlgenError> {
        let driver: Box<dyn TextEncoderDriver> = match &self.config {
            ModelConfig::OpenAIEmbeddings(cfg) => Box::new(OpenAIEmbeddingsDriver::new(cfg)?),
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
            ModelConfig::Stub(cfg) => Box::new(StubDriver::new(cfg)?),
            _ => return Err(SqlgenError::UnsupportedModelConfig(self.name.clone())),
        };

        Ok(driver)
    }

    pub fn get_text_instruct_model(&self) -> Result<Box<dyn TextInstructDriver>, SqlgenError> {
        let driver: Box<dyn TextInstructDriver> = match &self.config {
            ModelConfig::OpenAICompletions(cfg) => Box::new(OpenAICompletionsDriver::new(cfg)?),
            ModelConfig::Ollama(cfg) => Box::new(OllamaDriver::new(cfg)?),
            ModelConfig::Stub(cfg) => Box::new(StubDriver::new(cfg)?),
            _ => return Err(SqlgenError::UnsupportedModelConfig(self.name.clone())),
        };

        Ok(driver)
    }
}
