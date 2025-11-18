use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::types::{
    errors::SqlgenError,
    structs::{instruct_message::InstructMessage, profiles::OpenAICompletionsConfig},
    traits::driver::{ModelDriver, TextInstructDriver},
};

pub struct OpenAICompletionsDriver {
    config: OpenAICompletionsConfig,
    messages: Vec<InstructMessage>,
    client: Client,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAICompletionsRequest {
    model: String,
    messages: Vec<InstructMessage>,
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

    fn get_auth_header(&self) -> Option<String> {
        let api_key = match self.config.api_key.clone() {
            Some(k) => k,
            None => return None,
        };

        Some(
            format!("{} {}", self.config.authorization_type, api_key)
                .trim()
                .to_string(),
        )
    }
}

#[async_trait]
impl TextInstructDriver for OpenAICompletionsDriver {
    async fn get_assistant_response(
        &mut self,
        messages: Vec<InstructMessage>,
    ) -> Result<String, SqlgenError> {
        self.messages.extend(messages);

        let body = self.get_request_body()?;
        let url = self.config.url.clone();
        let auth_header = self.get_auth_header();
        let resp = self
            .client
            .post(url)
            .body(body)
            .header("Content-Type", "application/json");
        let resp = match auth_header {
            Some(h) => resp.header("Authorization", &h),
            None => resp,
        };

        unimplemented!();
    }
}
