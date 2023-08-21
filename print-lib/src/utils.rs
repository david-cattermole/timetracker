pub fn option_string_to_string(value: &Option<String>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}
