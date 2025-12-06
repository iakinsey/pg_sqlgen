use crate::{
    debug1,
    types::{
        errors::SqlgenError,
        structs::{instruct_message::InstructMessage, profiles::OllamaConfig},
        traits::driver::{ModelDriver, TextEncoderDriver, TextInstructDriver},
    },
};
use async_trait::async_trait;
use futures::{stream, StreamExt, TryStreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Value};

#[derive(Serialize)]
struct ChatBody {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    format: Option<Value>,
}

#[derive(Serialize)]
struct EmbeddingsBody<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
pub struct OllamaChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize, Debug)]
pub struct OllamaEmbeddingsResponse {
    pub embedding: Vec<f32>,
}

// Enables interaction with Ollama servers. Supports both chat and text
// encoding.
// https://ollama.com/
pub struct OllamaDriver {
    config: OllamaConfig,
    messages: Vec<InstructMessage>,
    client: Client,
}

impl ModelDriver for OllamaDriver {
    const ID: &'static str = "ollama";
    const NAME: &'static str = "Ollama";
    const DESCRIPTION: &'static str =
        "A flexible runtime for running and managing language models.";
}

impl OllamaDriver {
    pub fn new(config: &OllamaConfig) -> Result<Self, SqlgenError> {
        Ok(Self {
            config: config.clone(),
            messages: Vec::new(),
            client: Client::new(),
        })
    }

    fn get_encode_body(&self, input: &str) -> Result<String, SqlgenError> {
        let message = EmbeddingsBody {
            model: &self.config.model_name,
            prompt: input,
        };

        let json = to_string(&message)?;

        Ok(json)
    }

    fn get_chat_body(&self) -> Result<String, SqlgenError> {
        let messages: Vec<ChatMessage> = self
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role.as_string(),
                content: m.message.clone(),
            })
            .collect();

        let output_format = self
            .messages
            .iter()
            .rev()
            .find_map(|m| m.output_format.clone());

        let body = ChatBody {
            model: self.config.model_name.clone(),
            messages,
            stream: false,
            format: output_format,
        };

        let json = to_string(&body)?;

        Ok(json)
    }

    fn get_request_url(&self, method: &str) -> String {
        let scheme = match self.config.use_https {
            true => "https",
            false => "http",
        };

        format!(
            "{scheme}://{host}/api/{method}",
            scheme = scheme,
            host = self.config.host,
            method = method
        )
    }
}

#[async_trait]
impl TextEncoderDriver for OllamaDriver {
    async fn dimensions(&self) -> Result<usize, SqlgenError> {
        Ok(self.encode("a").await?.len())
    }

    async fn encode(&self, input: &str) -> Result<Vec<f32>, SqlgenError> {
        let url = self.get_request_url("embeddings");
        let body = self.get_encode_body(input)?;

        debug1!("OllamaDriver (encoder) request: {}", body);

        let resp = self.client.post(url).body(body.clone()).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(SqlgenError::ResponseError(format!(
                "HTTP {}: {}\nRequest:\n{}",
                status, text, body
            )));
        }

        match from_str::<OllamaEmbeddingsResponse>(&text) {
            Ok(response) => Ok(response.embedding),
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {}\nRequest:\n{}",
                status, text, body
            ))),
        }
    }

    async fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, SqlgenError> {
        let mut futures = Vec::with_capacity(inputs.len());

        for i in inputs {
            futures.push(async { self.encode(i).await });
        }

        stream::iter(futures)
            .buffer_unordered(self.config.request_batch_size)
            .try_collect()
            .await
    }
}

#[async_trait]
impl TextInstructDriver for OllamaDriver {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, SqlgenError> {
        self.messages.extend(messages);

        let url = self.get_request_url("chat");
        let body = self.get_chat_body()?;

        debug1!("OllamaDriver (instruct) request: {}", body);

        let resp = self.client.post(url).body(body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        debug1!("OllamaDriver (instruct) response: {}", text);

        if !status.is_success() {
            return Err(SqlgenError::ResponseError(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        match from_str::<OllamaChatResponse>(&text) {
            Ok(response) => Ok(response.message.content),
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {}",
                status, text
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::POST, MockServer};

    use crate::types::{
        formats::generate_output_format_schema, structs::instruct_message::InstructRole,
    };

    use super::*;

    #[tokio::test]
    async fn test_get_assistant_response() {
        let assistant_response = r#"{
            "model": "test-model",
            "message": {
                "role": "assistant",
                "content": "Test response" 
            }
        }"#;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/chat")
                .body_contains("test-model");

            then.status(200).body(assistant_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: "test".to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OllamaDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await.unwrap();

        mock.assert();
        assert_eq!(response, "Test response");
    }

    #[tokio::test]
    async fn test_get_assistant_response_failed() {
        let error_response = r#"{"error": "test"}"#;
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/chat")
                .body_contains("test-model");

            then.status(500).body(error_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: "test".to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OllamaDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await;

        mock.assert();

        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_get_assistant_response_parse_failed() {
        let bad_response = r#"}{"#;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/chat")
                .body_contains("test-model");

            then.status(200).body(bad_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: "test".to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OllamaDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await;

        mock.assert();

        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_encode_many() {
        let encoding_response = r#"{
            "embedding": [
                0.2139129936695099,
                0.05833360552787781
            ]
        }"#;
        let inputs = vec![
            "test-input-1",
            "test-input-2",
            "test-input-3",
            "test-input-4",
            "test-input-5",
        ];

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(200).body(encoding_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 2,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode_many(&inputs).await.unwrap();

        mock.assert_hits(inputs.len());

        assert_eq!(response.len(), inputs.len())
    }

    #[tokio::test]
    async fn test_encode_many_response_failed() {
        let error_response = r#"{"error": "test"}"#;
        let inputs = vec![
            "test-input-1",
            "test-input-2",
            "test-input-3",
            "test-input-4",
            "test-input-5",
        ];

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(500).body(error_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 2,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode_many(&inputs).await;

        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_encode_many_parse_failed() {
        let bad_response = "}{";
        let inputs = vec![
            "test-input-1",
            "test-input-2",
            "test-input-3",
            "test-input-4",
            "test-input-5",
        ];

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(200).body(bad_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 2,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode_many(&inputs).await;

        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_encode() {
        let encoding_response = r#"{
            "embedding": [
                0.2139129936695099,
                0.05833360552787781
            ]
        }"#;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(200).body(encoding_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode("test input").await.unwrap();

        mock.assert();

        assert_eq!(response.len(), 2)
    }

    #[tokio::test]
    async fn test_encode_response_failed() {
        let error_response = r#"{"error": "test"}"#;
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(500).body(error_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode("test input").await;

        mock.assert();

        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_encode_parse_failed() {
        let bad_response = r#"}{"#;
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(200).body(bad_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let response = driver.encode("test input").await;

        mock.assert();

        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_get_dimensions() {
        let encoding_response = r#"{
            "embedding": [
                0.2139129936695099,
                0.05833360552787781
            ]
        }"#;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/embeddings")
                .body_contains("test-model");

            then.status(200).body(encoding_response);
        });

        let config = OllamaConfig {
            host: format!("{}:{}", server.host(), server.port()),
            model_name: "test-model".to_string(),
            use_https: false,
            request_batch_size: 1,
        };

        let driver = OllamaDriver::new(&config).unwrap();
        let size = driver.dimensions().await.unwrap();

        mock.assert();
        assert_eq!(size, 2);
    }
}
