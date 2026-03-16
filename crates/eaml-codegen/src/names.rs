//! Name conversion utilities for EAML code generation.
//!
//! Converts EAML PascalCase/camelCase identifiers to Python-style
//! snake_case and UPPER_SNAKE_CASE names.

/// Converts a PascalCase or camelCase string to snake_case.
///
/// Handles sequences of uppercase letters (e.g., "HTTPClient" -> "http_client")
/// and mixed case (e.g., "getData" -> "get_data").
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                // Insert underscore before an uppercase char if:
                // 1. Previous char is lowercase (e.g., "getData" -> "get_Data")
                // 2. Previous char is uppercase AND next char is lowercase
                //    (e.g., "HTTPClient" -> "HTTP_Client")
                if prev.is_lowercase()
                    || (prev.is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase())
                {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

/// Converts a PascalCase or camelCase string to UPPER_SNAKE_CASE.
pub fn to_upper_snake_case(s: &str) -> String {
    to_snake_case(s).to_uppercase()
}

/// Converts a name to a config constant name (UPPER_SNAKE_CASE + "_CONFIG").
pub fn to_config_name(s: &str) -> String {
    format!("{}_CONFIG", to_upper_snake_case(s))
}
