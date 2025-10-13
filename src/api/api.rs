use pgrx::prelude::*;
use tokio::runtime::Runtime;

use crate::stores::{metadata_store::MetadataStore, model_store::ModelStore};

#[pg_extern]
fn certify_query() -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn add_model(
    model_name: &str,
    schema_name: &str,
    config_str: &str,
    generate_prompt: Option<&str>,
    filter_prompt: Option<&str>,
) {
    ModelStore::create_model_profile(model_name, schema_name, config_str)
        .unwrap_or_else(|e| error!("{}", e));

    /*
    let encoder =
        ModelStore::get_text_encoder_model(model_name).unwrap_or_else(|e| error!("{}", e));
    let rt = Runtime::new().unwrap_or_else(|e| error!("failed to initialize runtime: {}", e));

    rt.block_on(async {
        MetadataStore::initialize_metadata(model_name, schema_name, encoder)
            .await
            .unwrap_or_else(|e| error!("{}", e));
    })
    */
}

#[pg_extern]
fn remove_model(model_name: &str) {
    let profile = ModelStore::get_model_profile(model_name).unwrap_or_else(|e| error!("{}", e));
    // TODO dont allow a model to be removed if its used by an engine

    //MetadataStore::remove_metadata(model_name, &profile.schema).unwrap_or_else(|e| error!("{}", e));
    ModelStore::delete_model_profile(model_name).unwrap_or_else(|e| error!("{}", e));
}

#[pg_extern]
fn execute() -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn generate() -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn set_default_model() -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn create_engine(
    name: &str,
    instruct_model: &str,
    encoder_model: &str,
    generate_prompt: Option<&str>,
    filter_prompt: Option<&str>,
) {
    unimplemented!()
}

#[pg_extern]
fn remove_engine(name: &str) {
    unimplemented!()
}
