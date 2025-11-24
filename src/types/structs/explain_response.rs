use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainResponse {
    pub text: Option<String>,
    pub error: Option<String>,
}
