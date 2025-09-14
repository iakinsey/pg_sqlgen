use serde_json::Value;

pub struct ModelProfile {
    name: String,
    driver_name: String,
    config: Value,
}
