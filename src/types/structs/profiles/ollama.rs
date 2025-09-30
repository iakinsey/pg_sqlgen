use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub model_name: String,
    #[serde(default = "default_use_https")]
    pub use_https: bool,
}

fn default_host() -> String {
    "localhost:11434".to_string()
}

fn default_use_https() -> bool {
    false
}
