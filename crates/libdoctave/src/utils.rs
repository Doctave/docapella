/// Capitalize a str
pub(crate) fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Serde helper for checking if a HashMap is empty
#[allow(dead_code)]
pub(crate) fn empty_map<K, V>(map: &std::collections::HashMap<K, V>) -> bool {
    map.is_empty()
}

/// Serde helper for checking if a bool is false
#[allow(dead_code)]
pub(crate) fn is_false(val: &bool) -> bool {
    !(*val)
}
