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

#[derive(Serialize)]
pub struct EmbeddingsRequest<'a> {
    input: &'a str,
    model: &'a str,
    encoding_format: &'a str,
}

#[derive(Serialize)]
pub struct BatchEmbeddingsRequest<'a> {
    input: &'a [&'a str],
    model: &'a str,
    encoding_format: &'a str,
}

#[derive(Deserialize, Serialize)]
pub struct EmbeddingsResponse {
    data: Vec<EmbeddingsData>,
}

#[derive(Deserialize, Serialize)]
pub struct EmbeddingsData {
    embedding: Vec<f32>,
}

// Enables interaction with models that implement OpenAI's embeddings API. While
// the API was initially designed for use with OpenAI's models, the format is
// commonly used by other models.
// https://platform.openai.com/docs/api-reference/embeddings
pub struct OpenAIEmbeddingsDriver {
    config: OpenAIEmbeddingsConfig,
    client: Client,
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
        let request_json = EmbeddingsRequest {
            input: input,
            model: &self.config.model,
            encoding_format: "float",
        };
        let body = to_string(&request_json)?;
        let auth_header =
            get_auth_header(self.config.api_key.clone(), &self.config.authorization_type);
        let req = self
            .client
            .post(&self.config.url)
            .body(body.clone())
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
                "HTTP {}: {}\nRequest:\n{}",
                status, text, body,
            )));
        }

        match from_str::<EmbeddingsResponse>(&text) {
            Ok(response) => {
                let embedding = response
                    .data
                    .into_iter()
                    .next()
                    .ok_or_else(|| SqlgenError::ResponseError("no embeddings returned".into()))?
                    .embedding;

                Ok(embedding)
            }
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {}\nRequest:\n{}",
                status, text, body
            ))),
        }
    }

    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, SqlgenError> {
        let request_json = BatchEmbeddingsRequest {
            input: inputs,
            model: &self.config.model,
            encoding_format: "float",
        };
        let body = to_string(&request_json)?;
        let auth_header =
            get_auth_header(self.config.api_key.clone(), &self.config.authorization_type);
        let req = self
            .client
            .post(&self.config.url)
            .body(body.clone())
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
                "HTTP {}: {}\nRequest:\n{}",
                status, text, body
            )));
        }

        match from_str::<EmbeddingsResponse>(&text) {
            Ok(response) => Ok(response.data.into_iter().map(|d| d.embedding).collect()),
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {}\nRequest:\n{}",
                status, text, body
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::POST, MockServer};

    #[tokio::test]
    async fn test_encode_many() {
        let response = EmbeddingsResponse {
            data: vec![
                EmbeddingsData {
                    embedding: vec![0.1, 0.2, 0.3],
                },
                EmbeddingsData {
                    embedding: vec![1.1, 1.2, 1.3],
                },
                EmbeddingsData {
                    embedding: vec![2.1, 2.2, 2.3],
                },
            ],
        };
        let encode_response = to_string(&response).unwrap();
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let inputs = vec!["test-input-1", "test-input-2", "test-input-3"];
        let api_key = "api-key";
        let auth_type = "Bearer";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(*inputs.get(0).unwrap())
                .body_contains(*inputs.get(1).unwrap())
                .body_contains(*inputs.get(2).unwrap())
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
        let actual_response = driver.encode_many(&inputs).await.unwrap();

        mock.assert();
        assert_eq!(
            actual_response,
            response
                .data
                .iter()
                .map(|d| d.embedding.clone())
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_encode_many_response_failed() {
        let error_response = r#"{"error": "test"}"#;
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let inputs = vec!["test-input-1", "test-input-2", "test-input-3"];
        let api_key = "api-key";
        let auth_type = "Bearer";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(*inputs.get(0).unwrap())
                .body_contains(*inputs.get(1).unwrap())
                .body_contains(*inputs.get(2).unwrap())
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
        let response = driver.encode_many(&inputs).await;

        mock.assert();
        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_encode_many_parse_failed() {
        let bad_response = r#"}{"#;
        let url_part = "/v1/embeddings";
        let model_name = "test-model-name";
        let inputs = vec!["test-input-1", "test-input-2", "test-input-3"];
        let api_key = "api-key";
        let auth_type = "Bearer";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(url_part)
                .body_contains(model_name)
                .body_contains(*inputs.get(0).unwrap())
                .body_contains(*inputs.get(1).unwrap())
                .body_contains(*inputs.get(2).unwrap())
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
        let response = driver.encode_many(&inputs).await;

        mock.assert();

        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

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
        assert!(response.unwrap_err().to_string().contains(&expected_err));
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
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }
}
