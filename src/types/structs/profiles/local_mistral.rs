use crate::utils::model::default_compute_device;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMistralConfig {
    #[serde(default = "default_model_name")]
    pub model_name: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    #[serde(default = "default_config_filename")]
    pub config_filename: String,
    #[serde(default = "default_tokenizer_filename")]
    pub tokenizer_filename: String,
    #[serde(default = "default_weights_filenames")]
    pub weights_filenames: Vec<String>,
    #[serde(default = "default_compute_device")]
    pub compute_device: String,
}

fn default_model_name() -> String {
    "Mistral-7B-Instruct-v0.2".to_string()
}

fn default_revision() -> String {
    "b207367332321f8e44f96e224ef15bc607f4dbf0".to_string()
}

fn default_config_filename() -> String {
    "config.json".to_string()
}

fn default_tokenizer_filename() -> String {
    "tokenizer.json".to_string()
}

fn default_weights_filenames() -> Vec<String> {
    "model.safetensors".to_string()
}
