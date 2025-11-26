// Get the properly formatted auth value for an Authorization HTTP header.
pub fn get_auth_header(api_key: Option<String>, auth_type: &str) -> Option<String> {
    let api_key = match api_key {
        Some(k) => k,
        None => return None,
    };

    Some(format!("{} {}", auth_type, api_key).trim().to_string())
}
