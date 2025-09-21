use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use candle_core::DType;
use candle_nn::VarBuilder;
use candle_transformers::models::{phi, phi3::{Config, Model}};
use hf_hub::{api::sync::Api, Repo};
use serde_json::from_str;
use tokenizers::Tokenizer;

use crate::{types::{errors::ModelDriverError, structs::{instruct_message::{InstructMessage, InstructRole}, profiles::LocalPhiConfig}, traits::driver::TextInstructDriver}, utils::model::get_device};


pub struct LocalPhiInstructDriver {
    model: Model,
    tokenizer: Tokenizer,
    config: LocalPhiConfig,
}


impl LocalPhiInstructDriver {
    pub fn new(phi_config: LocalPhiConfig) -> Result<Self, ModelDriverError> {
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
        
        Ok(Self {
            model,
            tokenizer,
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
    fn get_assistant_response(&mut self, messages: Vec<InstructMessage>) -> Result<String, ModelDriverError> {
        unimplemented!()
    }
}