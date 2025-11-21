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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::POST, MockServer};

    #[tokio::test]
    async fn test_encode() {
        let embedding = vec![0.0023064255, -0.009327292, -0.0028842222];
        let embedding_json = to_string(&embedding).unwrap();
        let encode_response = format!(
            r#"{{
            "data": [{{
                "embedding": {}
            }}]
        }}"#,
            embedding_json
        );
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let input = "test-input";
        let api_key = "api-key";
        let auth_type = "Bearer";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(input)
                .header("Authorization", format!("{} {}", auth_type, api_key));

            then.status(200).body(encode_response);
        });
        let config = OpenAIEmbeddingsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), url_part),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: auth_type.to_string(),
        };
        let driver = OpenAIEmbeddingsDriver::new(&config).unwrap();
        let response = driver.encode(input).await.unwrap();

        mock.assert();
        assert_eq!(response, embedding);
    }

    #[tokio::test]
    async fn test_encode_response_failed() {
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let input = "test-input";
        let api_key = "api-key";
        let auth_type = "Bearer";
        let error_response = r#"{"error": "test"}"#;
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(input)
                .header("Authorization", format!("{} {}", auth_type, api_key));

            then.status(500).body(error_response);
        });
        let config = OpenAIEmbeddingsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), url_part),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: auth_type.to_string(),
        };
        let driver = OpenAIEmbeddingsDriver::new(&config).unwrap();
        let response = driver.encode(input).await;

        mock.assert();
        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert_eq!(response.unwrap_err().to_string(), expected_err);
    }

    #[tokio::test]
    async fn test_encode_parse_failed() {
        let bad_response = "}{";
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let input = "test-input";
        let api_key = "api-key";
        let auth_type = "Bearer";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(input)
                .header("Authorization", format!("{} {}", auth_type, api_key));

            then.status(200).body(bad_response);
        });
        let config = OpenAIEmbeddingsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), url_part),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: auth_type.to_string(),
        };
        let driver = OpenAIEmbeddingsDriver::new(&config).unwrap();
        let response = driver.encode(input).await;

        mock.assert();
        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert_eq!(response.unwrap_err().to_string(), expected_err);
    }
}
