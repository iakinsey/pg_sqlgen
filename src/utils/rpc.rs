use crate::types::{errors::SqlgenError, traits::driver::TextEncoderDriver};

// Get the properly formatted auth value for an Authorization HTTP header.
pub fn get_auth_header(api_key: Option<String>, auth_type: &str) -> Option<String> {
    let api_key = match api_key {
        Some(k) => k,
        None => return None,
    };

    Some(format!("{} {}", auth_type, api_key).trim().to_string())
}

pub async fn wrap_encode(
    encoder: Box<dyn TextEncoderDriver>,
    values: Vec<&str>,
) -> Result<Vec<Option<Vec<f32>>>, SqlgenError> {
    let mut non_empty_indices = Vec::new();
    let mut non_empty_values = Vec::new();

    for (idx, v) in values.iter().enumerate() {
        if !v.is_empty() {
            non_empty_indices.push(idx);
            non_empty_values.push(*v);
        }
    }

    let encoded_non_empty = match non_empty_values.is_empty() {
        true => vec![],
        false => encoder.encode_many(&non_empty_values).await?,
    };

    let mut out = vec![None; values.len()];

    for (pos_in_encoded, original_idx) in non_empty_indices.into_iter().enumerate() {
        out[original_idx] = Some(encoded_non_empty[pos_in_encoded].clone());
    }

    Ok(out)
}
