use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::{
    types::{
        errors::SqlgenError,
        structs::{instruct_message::InstructMessage, profiles::OpenAICompletionsConfig},
        traits::driver::{ModelDriver, TextInstructDriver},
    },
    utils::{rpc::get_auth_header, schema::get_prompt_with_schema},
};

#[derive(Deserialize, Serialize)]
pub struct OpenAICompletionsRequest {
    model: String,
    messages: Vec<CompletionsMessage>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CompletionsMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
pub struct OpenAICompletionsResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    message: Message,
}

#[derive(Deserialize)]
pub struct Message {
    content: String,
}

// Enables interaction with models that implement OpenAI's chat completions API.
// While the API was initially designed for use with OpenAI's models, the format
// is commonly used by other models.
// https://platform.openai.com/docs/api-reference/chat
pub struct OpenAICompletionsDriver {
    config: OpenAICompletionsConfig,
    messages: Vec<CompletionsMessage>,
    client: Client,
}

impl ModelDriver for OpenAICompletionsDriver {
    const ID: &'static str = "openai_completions";
    const NAME: &'static str = "OpenAI Completions";
    const DESCRIPTION: &'static str = "Models that implement OpenAI's chat completions API.";
}

impl OpenAICompletionsDriver {
    pub fn new(config: &OpenAICompletionsConfig) -> Result<Self, SqlgenError> {
        Ok(Self {
            config: config.clone(),
            messages: Vec::new(),
            client: Client::new(),
        })
    }

    fn get_request_body(&self) -> Result<String, SqlgenError> {
        let request = OpenAICompletionsRequest {
            model: self.config.model.clone(),
            messages: self.messages.clone(),
        };

        Ok(to_string(&request)?)
    }

    fn append_messages(&mut self, messages: Vec<InstructMessage>) {
        self.messages
            .extend(messages.iter().map(|m| CompletionsMessage {
                role: m.role.to_string().to_lowercase(),
                content: get_prompt_with_schema(m),
            }));
    }
}

#[async_trait]
impl TextInstructDriver for OpenAICompletionsDriver {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, SqlgenError> {
        self.append_messages(messages);

        let body = self.get_request_body()?;
        let url = self.config.url.clone();
        let auth_header =
            get_auth_header(self.config.api_key.clone(), &self.config.authorization_type);

        let req = self
            .client
            .post(url)
            .body(body.clone())
            .header("Content-Type", "application/json");

        let req = match auth_header {
            Some(h) => req.header("Authorization", &h),
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

        match from_str::<OpenAICompletionsResponse>(&text) {
            Ok(response) => {
                let first = response
                    .choices
                    .first()
                    .ok_or_else(|| SqlgenError::ResponseError("no choices returned".to_string()))?;
                Ok(first.message.content.clone())
            }
            Err(_) => Err(SqlgenError::ResponseError(format!(
                "HTTP {}, failed to parse: {} \nRequest:\n{}",
                status, text, body
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{
        formats::generate_output_format_schema, structs::instruct_message::InstructRole,
    };

    use super::*;
    use httpmock::{Method::POST, MockServer};

    #[tokio::test]
    async fn test_get_assistant_response() {
        let response_text = "test-response-text";
        let assistant_response = format!(
            r#"{{
            "choices": [{{
                "message": {{
                    "content": "{}"
                }}
            }}]
        }}"#,
            response_text
        );
        let model_name = "test-model";
        let model_path = "/v1/chat/completions";
        let server = MockServer::start();
        let api_key = "test-api-key";
        let authorization_type = "Bearer";
        let message = "test-message";
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(model_path)
                .body_contains(model_name)
                .body_contains(message)
                .header(
                    "Authorization",
                    format!("{} {}", authorization_type, api_key),
                );

            then.status(200).body(assistant_response);
        });
        let config = OpenAICompletionsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), model_path),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: authorization_type.to_string(),
        };
        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: message.to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OpenAICompletionsDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await.unwrap();

        mock.assert();
        assert_eq!(response, response_text);
    }

    #[tokio::test]
    async fn test_get_assistant_response_failed() {
        let model_name = "test-model";
        let model_path = "/v1/chat/completions";
        let api_key = "test-api-key";
        let authorization_type = "Bearer";
        let error_response = r#"{"error": "test"}"#;
        let message = "test-message";
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(model_path)
                .body_contains(model_name)
                .body_contains(message)
                .header(
                    "Authorization",
                    format!("{} {}", authorization_type, api_key),
                );

            then.status(500).body(error_response);
        });

        let config = OpenAICompletionsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), model_path),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: authorization_type.to_string(),
        };

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: message.to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OpenAICompletionsDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await;

        mock.assert();

        let expected_err = format!("HTTP 500 Internal Server Error: {}", error_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }

    #[tokio::test]
    async fn test_get_assistant_response_parse_failed() {
        let bad_response = "}{";
        let model_name = "test-model";
        let model_path = "/v1/chat/completions";
        let api_key = "test-api-key";
        let authorization_type = "Bearer";
        let message = "test-message";

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path(model_path)
                .body_contains(model_name)
                .body_contains(message)
                .header(
                    "Authorization",
                    format!("{} {}", authorization_type, api_key),
                );

            then.status(200).body(bad_response);
        });

        let config = OpenAICompletionsConfig {
            url: format!("http://{}:{}{}", server.host(), server.port(), model_path),
            model: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            authorization_type: authorization_type.to_string(),
        };

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: message.to_string(),
            output_format: Some(generate_output_format_schema()),
        }];

        let mut driver = OpenAICompletionsDriver::new(&config).unwrap();
        let response = driver.get_assistant_response(messages).await;

        mock.assert();

        let expected_err = format!("HTTP 200 OK, failed to parse: {}", bad_response);
        assert!(response.unwrap_err().to_string().contains(&expected_err));
    }
}
