use crate::{
    errors::ParseError,
    helpers::{parse_header_line::parse_header_line, parse_request_line::parse_request_line},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Headers {
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub fields: HashMap<String, String>,
}

impl Headers {
    pub fn parse(raw: &[u8]) -> Result<Self, ParseError> {
        let text = str::from_utf8(raw).map_err(|_| ParseError::InvalidUtf8)?;
        let text = text.strip_suffix("\r\n\r\n").unwrap_or(text);

        let mut lines = text.split("\r\n");
        let request_line = lines.next().ok_or(ParseError::MissingRequestLine)?;
        let (method, path, http_version) = parse_request_line(request_line)?;

        let mut fields = HashMap::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            let (name, value) = parse_header_line(line)?;
            fields.insert(name, value);
        }

        Ok(Self {
            method,
            path,
            http_version,
            fields,
        })
    }
}
