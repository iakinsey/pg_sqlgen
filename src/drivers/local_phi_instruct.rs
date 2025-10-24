use std::{fs::read_to_string, str::FromStr};

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::{generation::LogitsProcessor, models::phi3::{Config, Model}, utils::apply_repeat_penalty};
use hf_hub::{api::sync::Api, Repo};
use serde_json::from_str;
use tokenizers::Tokenizer;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::{types::{errors::SqlgenError, structs::{instruct_message::{InstructMessage, InstructRole}, profiles::LocalPhiConfig}, traits::driver::TextInstructDriver}, utils::model::get_device};


pub struct LocalPhiInstructDriver {
    model: Model,
    tokenizer: Tokenizer,
    eos_token: u32,
    config: LocalPhiConfig,
    device: Device,
    logits_processor: LogitsProcessor,
}

fn write_string_to_file<P: AsRef<Path>>(path: P, contents: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())
}

impl LocalPhiInstructDriver {
    pub fn new(phi_config: LocalPhiConfig) -> Result<Self, SqlgenError> {
        let self_config = phi_config.clone();
        let device = get_device(&phi_config.compute_device)?;
        let repo = Repo::with_revision(
            phi_config.model_name.clone(),
            hf_hub::RepoType::Model,
            phi_config.revision.clone(),
        );
        let api = Api::new()?.repo(repo);
        let tokenizer_file = api.get(&phi_config.tokenizer_filename)?;
        let tokenizer = Tokenizer::from_file(tokenizer_file)?;
        let config_file = api.get(&phi_config.config_filename)?;
        let config_json = read_to_string(config_file)?;
        let config: Config = from_str(&config_json)?;
        write_string_to_file("/home/agent/wat", "worked")?;
        let dtype = match phi_config.data_type {
            Some(s) => DType::from_str(&s)?,
            None => device.bf16_default_to_f32(),
        };
        let weights_files: Vec<_> = phi_config
            .weights_filenames
            .into_iter()
            .map(|weights_filename| api.get(&weights_filename))
            .collect::<Result<_, _>>()?;
        let var_builder = unsafe { VarBuilder::from_mmaped_safetensors(&weights_files, dtype, &device)? };
        let model = Model::new(&config, var_builder)?;
        let logits_processor = LogitsProcessor::new(phi_config.seed, phi_config.temperature, phi_config.top_p);
        let eos_token = tokenizer.get_vocab(true).get("<|endoftext|>").copied().ok_or(SqlgenError::Any("eos token does not exist".to_string()))?;
        
        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor,
            eos_token,
            config: self_config,
        })
    }

    pub fn gen_prompt(&self, messages: Vec<InstructMessage>) -> String {
        let mut prompt: String = messages
            .into_iter()
            .map(|m| {
                let role = match m.role {
                    InstructRole::System => "system",
                    InstructRole::User => "user",
                    InstructRole::Assistant => "assistant",
                };
                format!("<|{}|>{}<|end|>\n", role, m.message)
            })
            .collect();

        prompt.push_str("<|assistant|>");

        prompt
    }
}

impl TextInstructDriver for LocalPhiInstructDriver {
    async fn get_assistant_response(&mut self, messages: Vec<InstructMessage>) -> Result<String, SqlgenError> {
        let prompt = self.gen_prompt(messages);
        let tokens = self.tokenizer.encode(prompt, true)?.get_ids().to_vec();
        let mut pos = 0;
        let mut out_tokens = Vec::new();

        for index in 0..self.config.sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let ctx = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctx, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, pos)?.i((.., 0, ..))?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.config.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.config.repeat_last_n);
                apply_repeat_penalty(&logits, self.config.repeat_penalty, &tokens[start_at..])?
            };
            let next_token = self.logits_processor.sample(&logits)?;

            out_tokens.push(next_token);

            if next_token == self.eos_token {
                break;
            }

            pos += context_size;
        }

        Ok(match out_tokens.is_empty() {
            true => String::new(),
            false => self.tokenizer.decode(&out_tokens, true)?
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use crate::{drivers::LocalPhiInstructDriver, types::{structs::{instruct_message::{InstructMessage, InstructRole}, profiles::LocalPhiConfig}, traits::driver::TextInstructDriver}};

    fn setup_scenario() -> LocalPhiInstructDriver {
        let phi_config: LocalPhiConfig = from_str("{}").unwrap();
        LocalPhiInstructDriver::new(phi_config).unwrap()
    }

    #[test]
    fn test_generate_sql_statement() {
    }

    #[test]
    fn test_genrate_sentence() {}

    #[test]
    fn test_question_answer() {
        let messages = vec![
            InstructMessage {
                role: InstructRole::System,
                message: "You are a helpful agent, you answer questions.".to_string(),
            },
            InstructMessage {
                role:InstructRole::User,
                message: "What colors are on the flag of the United States?".to_string(),
            }
        ];

        let mut model = setup_scenario();
        let response = model.get_assistant_response(messages).unwrap();

        assert_eq!(response, "hello world");
    }
}