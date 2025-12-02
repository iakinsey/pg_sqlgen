use serde_json::{json, Value};

pub fn generate_output_format_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
             "reasoning": {
                "type": "string",
                "description": "Reasoning about what query to generate."
            },
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
             "reasoning": {
                "type": "string",
                "description": "Reasoning about what explanation to generate."
            },
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
             "reasoning": {
                "type": "string",
                "description": "Reasoning about what columns to filter."
            },
            "ddls": {
                "type": "array",
                "items": { "type": "string" },
                "description": "A list of relevant columns."
            },
            "error": {
                "type": "string",
                "description": "Optional error message if column filtering fails."
            }
        },
        "required": ["ddls"]
    })
}
