use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OllamaConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub model_name: String,
    #[serde(default = "default_use_https")]
    pub use_https: bool,
    #[serde(default = "default_request_batch_size")]
    pub request_batch_size: usize,
}

fn default_host() -> String {
    "localhost:11434".to_string()
}

fn default_use_https() -> bool {
    false
}

fn default_request_batch_size() -> usize {
    64
}
