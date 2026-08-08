use std::collections::HashMap;

pub fn split_path_and_query(target: &str) -> (String, HashMap<String, String>) {
    let mut query = HashMap::new();
    let (path, query_str) = match target.split_once('?') {
        Some((path, query_str)) => (path, Some(query_str)),
        None => (target, None),
    };

    if let Some(query_str) = query_str {
        for pair in query_str.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((key, value)) => (key, value),
                None => (pair, ""),
            };
            query.insert(key.to_string(), value.to_string());
        }
    }

    (path.to_string(), query)
}
