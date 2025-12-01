use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DDLResponse {
    pub ddls: Option<Vec<String>>,
    pub error: Option<String>,
}
