use serde_json::Value;

use crate::types::structs::instruct_message::InstructMessage;

// Converts a JSON schema into a prompt segment with output format instructions
pub fn schema_to_prompt_segment(schema: &Value) -> String {
    let mut out = String::new();
    let header = "You only respond as a json dictionary with the following keys:";
    out.push_str(header);
    out.push('\n');
    render_fields(schema, 1, &mut out);
    out
}

pub fn get_prompt_with_schema(message: &InstructMessage) -> String {
    match message.output_format.clone() {
        Some(f) => {
            let segment = schema_to_prompt_segment(&f);
            format!("{}\n\n{}", message.message, segment)
        }
        None => message.message.clone(),
    }
}

fn type_label(schema: &Value) -> String {
    match schema.get("type").and_then(|t| t.as_str()) {
        Some("string") => "string".into(),
        Some("number") => "number".into(),
        Some("integer") => "integer".into(),
        Some("boolean") => "boolean".into(),
        Some("object") => "object".into(),
        Some("array") => {
            if let Some(items) = schema.get("items") {
                match items.get("type").and_then(|t| t.as_str()) {
                    Some("object") => "list of objects".into(),
                    Some("string") => "list of strings".into(),
                    Some(t) => format!("list of {}", t),
                    None => "list".into(),
                }
            } else {
                "list".into()
            }
        }
        _ => "value".into(),
    }
}

fn render_fields(schema: &Value, indent: usize, out: &mut String) {
    let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };

    for (name, field_schema) in props {
        let tlabel = type_label(field_schema);
        let desc = field_schema
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        let pad = " ".repeat(indent);

        if desc.is_empty() {
            out.push_str(&format!("{pad}- {name} ({tlabel})\n"));
        } else {
            out.push_str(&format!("{pad}- {name} ({tlabel}) : {desc}\n"));
        }

        if field_schema.get("type").and_then(|t| t.as_str()) == Some("object") {
            render_fields(field_schema, indent + 2, out);
        }

        if field_schema.get("type").and_then(|t| t.as_str()) == Some("array") {
            if let Some(items) = field_schema.get("items") {
                if items.get("type").and_then(|t| t.as_str()) == Some("object") {
                    render_fields(items, indent + 2, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::formats::generate_output_format_schema, utils::schema::schema_to_prompt_segment,
    };

    #[test]
    fn test_parse_json_schema() {
        let prompt = schema_to_prompt_segment(&generate_output_format_schema());
        let expected_output = r#"You only respond as a json dictionary with the following keys:
 - error (string) : Optional error message if query generation fails.
 - query (string) : The generated SQL query.
"#;

        assert_eq!(prompt, expected_output);
    }
}
