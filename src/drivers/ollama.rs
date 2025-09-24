use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::types::{
    errors::ModelDriverError,
    structs::{instruct_message::InstructMessage, profiles::OllamaConfig},
    traits::driver::{TextEncoderDriver, TextInstructDriver},
};

#[derive(Serialize)]
struct ChatBody {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

pub struct OllamaDriver {
    config: OllamaConfig,
    messages: Vec<InstructMessage>,
    client: Client,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub message: ChatMessage,
}

impl OllamaDriver {
    pub fn new(config: OllamaConfig) -> Result<Self, ModelDriverError> {
        Ok(Self {
            config,
            messages: Vec::new(),
            client: Client::new(),
        })
    }

    fn get_chat_body(&self) -> Result<String, ModelDriverError> {
        let messages: Vec<ChatMessage> = self
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role.to_string(),
                content: m.message.clone(),
            })
            .collect();

        let body = ChatBody {
            model: self.config.model_name.clone(),
            messages,
            stream: false,
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

impl TextEncoderDriver for OllamaDriver {
    fn dimensions(&self) -> Result<usize, ModelDriverError> {
        unimplemented!()
    }

    fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError> {
        unimplemented!()
    }

    fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, ModelDriverError> {
        unimplemented!()
    }
}

impl TextInstructDriver for OllamaDriver {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, ModelDriverError> {
        self.messages.extend(messages);

        let url = self.get_request_url("chat");
        let body = self.get_chat_body()?;
        let resp = self.client.post(url).json(&body).send().await?;
        let text = resp.text().await?;
        let response: OllamaResponse = from_str(&text)?;

        Ok(response.message.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_assistant_response() {
        let assistant_response = r#"{{
            "model": "test-model",
            "message": {{
                "role": "assistant",
                "content": "Test response" 
            }}
        }}"#;
    }
}
