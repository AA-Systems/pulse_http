use std::collections::HashMap;

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::helpers::reason_phrase::reason_phrase;

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn text(status: u16, body: impl Into<String>) -> Self {
        let body = body.into().into_bytes();
        let mut response = Self::new(status);

        response
            .headers
            .insert("content-type".into(), "text/plain; charset=utf-8".into());

        response
            .headers
            .insert("content-length".into(), body.len().to_string());

        response.body = body;
        response
    }

    pub fn ok(body: impl Into<String>) -> Self {
        Self::text(200, body)
    }

    pub fn bad_request() -> Self {
        Self::text(400, "Bad request")
    }

    pub fn not_found() -> Self {
        Self::text(404, "Not found")
    }

    pub fn internal_server_error() -> Self {
        Self::text(500, "Internal server error")
    }

    pub async fn write_to_stream(self, stream: &mut TcpStream) {
        let reason = reason_phrase(self.status);
        let mut output = format!("HTTP/1.1 {} {} \r\n", self.status, reason);

        let mut has_content_length = false;
        for (name, value) in &self.headers {
            if name.eq_ignore_ascii_case("content-length") {
                has_content_length = true;
            }
            output.push_str(name);
            output.push_str(": ");
            output.push_str(value);
            output.push_str("\r\n");
        }

        if !has_content_length {
            output.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }
        output.push_str("\r\n");

        let _ = stream.write_all(output.as_bytes()).await;
        if !self.body.is_empty() {
            let _ = stream.write_all(&self.body).await;
        }
    }
}
