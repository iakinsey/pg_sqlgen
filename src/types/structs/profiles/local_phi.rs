use crate::{
    utils::model::default_compute_device,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub data_type: Option<String>,

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