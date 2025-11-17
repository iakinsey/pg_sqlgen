use crate::{types::errors::SqlgenError, utils::model::default_compute_device};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalPhiConfig {
    #[serde(default = "default_model_name")]
    pub model_name: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    #[serde(default = "default_compute_device")]
    pub compute_device: String,
    #[serde(default = "default_tokenizer_filename")]
    pub tokenizer_filename: String,
    #[serde(default = "default_config_filename")]
    pub config_filename: String,
    #[serde(default = "default_weights_filenames")]
    pub weights_filenames: Vec<String>,
    #[serde(default = "default_sample_len")]
    pub sample_len: usize,
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: String,
    #[serde(default = "default_repeat_last_n")]
    pub repeat_last_n: usize,
    #[serde(default = "default_seed")]
    pub seed: u64,

    pub temperature: Option<String>,
    pub top_p: Option<String>,
    pub data_type: Option<String>,
}

impl LocalPhiConfig {
    // Some values stored as String to allow PartialEq/Eq
    pub fn get_temperature(&self) -> Option<f64> {
        self.temperature
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
    }

    pub fn get_top_p(&self) -> Option<f64> {
        self.top_p.as_ref().and_then(|s| s.parse::<f64>().ok())
    }

    pub fn get_repeat_penalty(&self) -> Result<f32, SqlgenError> {
        Ok(self.repeat_penalty.parse::<f32>()?)
    }
}

fn default_model_name() -> String {
    "microsoft/Phi-4-mini-instruct".to_string()
}

fn default_revision() -> String {
    "main".to_string()
}

fn default_tokenizer_filename() -> String {
    "tokenizer.json".to_string()
}

fn default_config_filename() -> String {
    "config.json".to_string()
}

fn default_weights_filenames() -> Vec<String> {
    vec![
        "model-00001-of-00002.safetensors".to_string(),
        "model-00002-of-00002.safetensors".to_string(),
    ]
}

fn default_sample_len() -> usize {
    5000
}

fn default_repeat_penalty() -> String {
    "1.1".to_string()
}

fn default_repeat_last_n() -> usize {
    64
}

fn default_seed() -> u64 {
    5544542107816782714
}
