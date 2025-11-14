use pgrx::pg_extern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubConfig {
    #[serde(default = "default_instruct_output")]
    pub instruct_output: String,
    #[serde(default = "default_encode_output")]
    pub encode_output: Vec<f32>,
}

fn default_instruct_output() -> String {
    "stub".to_string()
}

fn default_encode_output() -> Vec<f32> {
    vec![0.0, 0.0, 0.0]
}

impl PartialEq for StubConfig {
    fn eq(&self, other: &Self) -> bool {
        self.instruct_output == other.instruct_output
            && self.encode_output.len() == other.encode_output.len()
            && self
                .encode_output
                .iter()
                .zip(&other.encode_output)
                .all(|(a, b)| (a - b).abs() < 1e-6)
    }
}

impl Eq for StubConfig {}
