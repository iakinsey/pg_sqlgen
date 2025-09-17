// BERT

use std::fs::read_to_string;

use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo};
use serde_json::from_str;
use tokenizers::Tokenizer;

use crate::{
    types::{
        errors::ModelDriverError,
        structs::profiles::LocalBertConfig,
        traits::driver::{ModelDriver, TextEncoderDriver},
    },
    utils::{math::l2_norm, model::get_device},
};

pub struct LocalBertDriver {
    model: BertModel,
    tokenizer: Tokenizer,
    hidden_size: usize,
    config: LocalBertConfig,
}

impl ModelDriver for LocalBertDriver {
    const NAME: &'static str = "Local BERT";
    const DESCRIPTION: &'static str = "A classic and lightweight text encoder that runs locally.";
}

impl LocalBertDriver {
    pub fn new(bert_config: &LocalBertConfig) -> Result<Self, ModelDriverError> {
        let device = get_device(&bert_config.compute_device)?;
        let repo = Repo::with_revision(
            bert_config.model_name.clone(),
            hf_hub::RepoType::Model,
            bert_config.revision.clone(),
        );
        let api = Api::new()?.repo(repo);
        let config_file = api.get(&bert_config.config_filename)?;
        let tokenizer_file = api.get(&bert_config.tokenizer_filename)?;
        let weights_file = api.get(&bert_config.weights_filename)?;
        let config_json = read_to_string(config_file)?;
        let config: Config = from_str(&config_json)?;
        let tokenizer = Tokenizer::from_file(tokenizer_file)?;
        let var_builder =
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_file], DTYPE, &device)? };
        let model = BertModel::load(var_builder, &config)?;

        Ok(LocalBertDriver {
            config: bert_config.clone(),
            model,
            tokenizer,
            hidden_size: config.hidden_size,
        })
    }
}

impl TextEncoderDriver for LocalBertDriver {
    fn dimensions(&self) -> Result<usize, ModelDriverError> {
        Ok(self.hidden_size)
    }

    fn encode(&self, input: &str) -> Result<Vec<f32>, ModelDriverError> {
        self.encode_many(&[input]).and_then(|mut results| {
            results
                .pop()
                .ok_or_else(|| ModelDriverError::EncodeError("no output".into()))
        })
    }

    fn encode_many(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, ModelDriverError> {
        // Largely ripped from https://github.com/huggingface/candle/blob/main/candle-examples/examples/bert/main.rs
        let tokens = self.tokenizer.encode_batch(inputs.to_vec(), true)?;
        let device = get_device(&self.config.compute_device)?;
        let (token_ids, attention_mask) = tokens.iter().try_fold(
            (Vec::new(), Vec::new()),
            |(mut token_ids, mut attention_mask), t| {
                let ids_tensor = Tensor::new(t.get_ids().to_vec().as_slice(), &device)?;
                let attn_tensor = Tensor::new(t.get_attention_mask().to_vec().as_slice(), &device)?;

                token_ids.push(ids_tensor);
                attention_mask.push(attn_tensor);

                Ok::<(Vec<Tensor>, Vec<Tensor>), ModelDriverError>((token_ids, attention_mask))
            },
        )?;

        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;
        let token_type_ids = token_ids.zeros_like()?;
        let embeddings = self
            .model
            .forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        let (_, n_tokens, _) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
        let embeddings = l2_norm(&embeddings)?;

        (0..inputs.len())
            .map(|i| Ok(embeddings.get(i)?.to_vec1::<f32>()?))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_json::from_str;

    use crate::{
        drivers::LocalBertDriver,
        types::{structs::profiles::LocalBertConfig, traits::driver::TextEncoderDriver},
        utils::math::cosine_similarity,
    };

    fn setup_scenario() -> LocalBertDriver {
        let bert_config: LocalBertConfig = from_str("{}").unwrap();

        LocalBertDriver::new(&bert_config).unwrap()
    }

    #[test]
    fn test_encode_success() {
        let examples = vec![
            "The quick brown fox jumps over the lazy dog",
            "Quick zephyrs blow, vexing daft Jim",
            "Bright vixens jump; dozy fowl quack.",
            "John quickly extemporized five tow bags.",
            "By Jove, my quick study of lexicography won a prize!",
        ];
        let model = setup_scenario();
        let encodings = model.encode_many(&examples).unwrap();
        let mut unique_set = HashSet::new();

        for encoding in encodings {
            assert_eq!(encoding.len(), model.dimensions().unwrap());
            let hex_encoding = encoding
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            unique_set.insert(hex_encoding);
        }

        assert_eq!(unique_set.len(), examples.len())
    }

    #[test]
    fn test_encode_many_success() {
        let sentence = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
        let model = setup_scenario();
        let encoding = model.encode(sentence).unwrap();

        assert_eq!(encoding.len(), model.dimensions().unwrap());
    }

    #[test]
    fn test_embeddings_distance() {
        let scenarios: Vec<[&str; 3]> = vec![
            ["emperor", "king", "keyboard"],
            ["cat", "dog", "planet"],
            ["iron", "steel", "david"],
            ["yellow", "red", "joystick"],
        ];
        let model = setup_scenario();

        for scenario in scenarios {
            let e = model.encode_many(&scenario).unwrap();
            let (closer_1, closer_2, farther) = (e[0].clone(), e[1].clone(), e[2].clone());
            let closer = cosine_similarity(&closer_1, &closer_2);
            let farther_1 = cosine_similarity(&closer_1, &farther);
            let farther_2 = cosine_similarity(&closer_2, &farther);

            assert!(
                closer > farther_1,
                "1: ({}, {}) > ({}, {})",
                scenario[0],
                scenario[1],
                scenario[0],
                scenario[2]
            );
            assert!(
                closer > farther_2,
                "2: ({}, {}) > ({}, {})",
                scenario[0],
                scenario[1],
                scenario[1],
                scenario[2]
            );
        }
    }
}
