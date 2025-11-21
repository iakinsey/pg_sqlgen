use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAIEmbeddingsConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_authorization_type")]
    pub authorization_type: String,
    pub api_key: Option<String>,
}

fn default_url() -> String {
    "https://api.openai.com/v1/embeddings".to_string()
}

fn default_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_authorization_type() -> String {
    "Bearer".to_string()
}
