use serde_json::Value;

pub struct ModelProfile {
    pub name: String,
    pub driver_name: String,
    pub profile: Value,
}
