use crate::{
    body::Body,
    helpers::{reason_phrase::reason_phrase, write_with_timeout::write_with_timeout},
};
use serde::Serialize;
use serde_json::to_vec;
use std::{
    collections::HashMap,
    fs::metadata,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncReadExt, net::TcpStream};

const FILE_READ_CHUNK: usize = 64 * 1024;

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Body::Bytes(Vec::new()),
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

        response.body = Body::Bytes(body);
        response
    }

    pub fn json(status: u16, body: impl Serialize) -> Self {
        let body = to_vec(&body).unwrap_or_default();
        let mut response = Self::new(status);

        response
            .headers
            .insert("content-type".into(), "application/json".into());

        response
            .headers
            .insert("content-length".into(), body.len().to_string());

        response.body = Body::Bytes(body);
        response
    }

    pub fn file(path: impl AsRef<Path>) -> Result<Self, ()> {
        let path = path.as_ref();
        let meta = metadata(path).map_err(|_| ())?;
        if !meta.is_file() {
            return Err(());
        }

        let len = meta.len();
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("download");

        let mut response = Self::new(200);
        response
            .headers
            .insert("content-type".into(), content_type_for(path).into());
        response
            .headers
            .insert("content-length".into(), len.to_string());
        response.headers.insert(
            "content-disposition".into(),
            format!("attachment; filename=\"{filename}\""),
        );
        response.body = Body::File {
            path: PathBuf::from(path),
            len,
        };
        Ok(response)
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

    pub fn too_many_requests() -> Self {
        Self::text(429, "Too many requests")
    }

    pub fn no_content() -> Self {
        let mut response = Self::new(204);
        response.headers.insert("content-length".into(), "0".into());
        response
    }

    pub fn internal_server_error() -> Self {
        Self::text(500, "Internal server error")
    }

    pub fn apply_security_headers(&mut self) {
        self.headers
            .entry("cache-control".into())
            .or_insert_with(|| "no-store".into());
        self.headers
            .entry("referrer-policy".into())
            .or_insert_with(|| "no-referrer".into());
        self.headers
            .entry("x-content-type-options".into())
            .or_insert_with(|| "nosniff".into());
        self.headers
            .entry("x-frame-options".into())
            .or_insert_with(|| "DENY".into());
    }

    pub async fn write_to_stream(
        mut self,
        write_timeout_sec: u64,
        stream: &mut TcpStream,
    ) -> Result<(), ()> {
        self.apply_security_headers();

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

        write_with_timeout(stream, output.as_bytes(), write_timeout_sec).await?;
        self.write_body(write_timeout_sec, stream).await
    }

    async fn write_body(self, write_timeout_sec: u64, stream: &mut TcpStream) -> Result<(), ()> {
        match self.body {
            Body::Bytes(bytes) => {
                if !bytes.is_empty() {
                    write_with_timeout(stream, &bytes, write_timeout_sec).await?;
                }
                Ok(())
            }
            Body::File { path, len } => stream_file(path, len, write_timeout_sec, stream).await,
        }
    }
}

async fn stream_file(
    path: PathBuf,
    len: u64,
    write_timeout_sec: u64,
    stream: &mut TcpStream,
) -> Result<(), ()> {
    let mut file = File::open(path).await.map_err(|_| ())?;
    let mut buf = vec![0u8; FILE_READ_CHUNK];
    let mut remaining = len;

    while remaining > 0 {
        let n = file.read(&mut buf).await.map_err(|_| ())?;
        if n == 0 {
            return Err(());
        }
        write_with_timeout(stream, &buf[..n], write_timeout_sec).await?;
        remaining = remaining.saturating_sub(n as u64);
    }

    Ok(())
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("txt" | "md") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}
