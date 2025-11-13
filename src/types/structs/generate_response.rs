use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateResponse {
    pub query: Option<String>,
    pub error: Option<String>,
}
