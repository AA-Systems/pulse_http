use crate::headers::Headers;

pub fn should_keep_alive(headers: &Headers) -> bool {
    if let Some(connection) = headers.fields.get("connection") {
        let value = connection.to_ascii_lowercase();
        if value == "close" {
            return false;
        }
        if value == "keep-alive" {
            return true;
        }
    }

    headers.http_version == "HTTP/1.1"
}
