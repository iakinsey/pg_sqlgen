use pgrx::pg_extern;

// This function lives in sqlgen_internal
#[pg_extern]
fn encode_text(model: &str, value: &str) -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn encode_text_batch(model: &str, values: Vec<String>) -> &'static str {
    unimplemented!()
}
