use crate::request::Request;

pub fn should_keep_alive(request: &Request) -> bool {
    if let Some(connection) = request.headers.get("connection") {
        let value = connection.to_ascii_lowercase();
        if value == "close" {
            return false;
        }
        if value == "keep-alive" {
            return true;
        }
    }

    request.http_version == "HTTP/1.1"
}
