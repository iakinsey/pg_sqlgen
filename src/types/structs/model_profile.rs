use crate::types::structs::profiles::{LocalBertConfig};

pub struct ModelProfile {
    pub name: String,
    pub driver_name: String,
    pub config: ModelConfig,
}

pub enum ModelConfig {
    LocalBert(LocalBertConfig),
}
