use pgrx::pg_extern;

// These functions lives in sqlgen_internal
#[pg_extern]
fn encode_text(model: &str, value: &str) -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn encode_text_batch(model: &str, values: Vec<String>) -> Vec<&'static str> {
    unimplemented!()
}

#[pg_extern]
fn decode_text(model: &str, value: &str) -> &'static str {
    unimplemented!()
}

#[pg_extern]
fn decode_text_batch(model: &str, values: Vec<String>) -> Vec<&'static str> {
    unimplemented!()
}
