use pgrx::pg_extern;
use pgrx::prelude::*;
use tokio::runtime::Runtime;

use crate::stores::model_store::ModelStore;
use crate::types::structs::instruct_message::InstructMessage;
use crate::types::structs::instruct_message::InstructRole;

// These functions lives in sqlgen_internal

#[pg_extern]
fn internal_encode_text(model: &str, text_value: &str) -> Vec<f32> {
    let profile = ModelStore::get_model_profile(model).unwrap_or_else(|e| error!("{}", e));
    let model = profile
        .get_text_encoder_model()
        .unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async { model.encode(text_value).await })
        .unwrap_or_else(|e| error!("{}", e))
}

#[pg_extern]
fn internal_batch_text_encode(model: &str, values: Vec<String>) -> Vec<Vec<f32>> {
    let profile = ModelStore::get_model_profile(model).unwrap_or_else(|e| error!("{}", e));
    let model = profile
        .get_text_encoder_model()
        .unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));
    let values: Vec<&str> = values.iter().map(|s| s.as_str()).collect();

    rt.block_on(async { model.encode_many(&values).await })
        .unwrap_or_else(|e| error!("{}", e))
}

/*
#[pg_extern]
fn internal_instruct_text(model: &str, system_prompt: &str, user_prompt: &str) -> String {
    let profile = ModelStore::get_model_profile(model).unwrap_or_else(|e| error!("{}", e));
    let mut model = profile
        .get_text_instruct_model()
        .unwrap_or_else(|e| error!("{}", e));
    let messages = vec![
        InstructMessage {
            role: InstructRole::System,
            message: system_prompt.to_string(),
        },
        InstructMessage {
            role: InstructRole::User,
            message: user_prompt.to_string(),
        },
    ];
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async { model.get_assistant_response(messages).await })
        .unwrap_or_else(|e| error!("{}", e))
}
*/

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use crate::pg_test;

    #[pg_test]
    fn test_internal_decode_text() {
        unimplemented!()
    }
}
