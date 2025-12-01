use serde_json::{json, Value};

pub fn generate_output_format_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The generated SQL query."
            },
            "error": {
                "type": "string",
                "description": "Optional error message if query generation fails."
            }
        },
        "required": ["query"]
    })
}

pub fn explain_output_format_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "Query description format."
            },
            "error": {
                "type": "string",
                "description": "Optional error message if response generation fails."
            }
        },
        "required": ["text"]
    })
}

pub fn ddl_output_format_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "ddls": {
                "type": "array",
                "items": { "type": "string" },
                "description": "A list of relevant DDLs."
            },
            "error": {
                "type": "string",
                "description": "Optional error message if DDL generation fails."
            }
        },
        "required": ["ddls"]
    })
}
