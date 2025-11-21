use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::{
    types::{
        errors::SqlgenError,
        structs::profiles::OpenAIEmbeddingsConfig,
        traits::driver::{ModelDriver, TextEncoderDriver},
    },
    utils::rpc::get_auth_header,
};

pub struct OpenAIEmbeddingsDriver {
    config: OpenAIEmbeddingsConfig,
    client: Client,
}

#[derive(Serialize)]
pub struct EmbeddingsRequest {
    input: String,
    model: String,
    encoding_format: String,
}

#[derive(Deserialize)]
pub struct EmbeddingsResponse {
    data: Vec<EmbeddingsData>,
}

#[derive(Deserialize)]
pub struct EmbeddingsData {
    embedding: Vec<f32>,
}

impl OpenAIEmbeddingsDriver {
    pub fn new(config: &OpenAIEmbeddingsConfig) -> Result<Self, SqlgenError> {
        Ok(Self {
            config: config.clone(),
            client: Client::new(),
        })
    }
}

impl ModelDriver for OpenAIEmbeddingsDriver {
    const ID: &'static str = "openai_embeddings";
    const NAME: &'static str = "OpenAI Embeddings";
    const DESCRIPTION: &'static str = "Models that implement OpenAI's embeddings API.";
}

#[async_trait]
impl TextEncoderDriver for OpenAIEmbeddingsDriver {
    async fn dimensions(&self) -> Result<usize, SqlgenError> {
        Ok(self.encode("a").await?.len())
    }

    async fn encode(&self, input: &str) -> Result<Vec<f32>, SqlgenError> {
        if input == "" {
            return Ok(vec![]);
        }

        let request_json = EmbeddingsRequest {
            input: input.to_string(),
            model: self.config.model.clone(),
            encoding_format: "float".to_string(),
        };
        let body = to_string(&request_json)?;
        let auth_header = get_auth_header(
            self.config.api_key.clone(),
            self.config.authorization_type.clone(),
        );
        let req = self
            .client
            .post(self.config.url.clone())
            .body(body)
            .header("Content-Type", "application/json");

        let req = match auth_header {
            Some(k) => req.header("Authorization", k),
            None => req,
        };
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(SqlgenError::ResponseError(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        match from_str::<EmbeddingsResponse>(&text) {
            Ok(response) => {
                let data = response.data.first().ok_or_else(|| {
                    SqlgenError::ResponseError("no embeddings returned".to_string())
                })?;

                Ok(data.embedding.clone())
            }
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {}",
                status, text
            ))),
        }
    }

    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, SqlgenError> {
        unimplemented!();
    }
}
