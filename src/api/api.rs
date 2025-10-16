use pgrx::prelude::*;
use tokio::runtime::Runtime;

use crate::{
    stores::{engine_store::EngineStore, metadata_store::MetadataStore, model_store::ModelStore},
    utils::sql::get_current_schema,
};

#[pg_extern]
fn add_model(model_name: &str, schema_name: &str, config_str: &str) {
    ModelStore::create_model_profile(model_name, schema_name, config_str)
        .unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern]
fn remove_model(model_name: &str) {
    ModelStore::delete_model_profile(model_name).unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern]
fn execute() -> &'static str {
    // TODO start here, probably need to fill this in under the engine mechanism?
    unimplemented!()
}

#[pg_extern]
fn generate(prompt: &str, engine: Option<&str>) -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn set_default_engine() -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn create_engine(
    name: &str,
    instruct_model: &str,
    encoder_model: &str,
    schema_name: Option<&str>,
    generate_prompt: Option<&str>,
    filter_prompt: Option<&str>,
) {
    let schema_name = match schema_name {
        Some(s) => s.to_string(),
        None => get_current_schema().unwrap_or_else(|e| error!("{}", e)),
    };

    ModelStore::get_model_profile(instruct_model).unwrap_or_else(|e| error!("{}", e));
    let encoder =
        ModelStore::get_text_encoder_model(encoder_model).unwrap_or_else(|e| error!("{}", e));

    EngineStore::create_engine(
        name,
        &schema_name,
        encoder_model,
        instruct_model,
        generate_prompt,
        filter_prompt,
    )
    .unwrap_or_else(|e| error!("{}", e));

    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async {
        MetadataStore::initialize_metadata(encoder_model, &schema_name, encoder)
            .await
            .unwrap_or_else(|e| error!("{}", e));
    })
}

#[pg_extern]
fn remove_engine(name: &str) {
    let engine = EngineStore::get_engine(name).unwrap_or_else(|e| error!("{}", e));

    MetadataStore::remove_metadata(&engine.encoder_model, &engine.schema_name)
        .unwrap_or_else(|e| error!("{}", e));
    EngineStore::remove_engine(name).unwrap_or_else(|e| error!("{}", e));
}
