use crate::types::structs::profiles::{LocalBertConfig, OllamaConfig};

pub struct ModelProfile {
    pub name: String,
    pub driver_name: String,
    pub config: ModelConfig,
}

pub enum ModelConfig {
    LocalBert(LocalBertConfig),
    Ollama(OllamaConfig),
}
