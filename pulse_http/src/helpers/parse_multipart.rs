use crate::{errors::MultipartError, helpers::find_subslice::find_subslice};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MultipartFile {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Multipart {
    pub fields: HashMap<String, String>,
    pub files: Vec<MultipartFile>,
}

pub fn parse_multipart(body: &[u8], content_type: &str) -> Result<Multipart, MultipartError> {
    let boundary = extract_boundary(content_type).ok_or(MultipartError::MissingBoundary)?;
    let marker = format!("--{boundary}");
    let marker_bytes = marker.as_bytes();
    let end_marker = format!("\r\n--{boundary}");
    let end_marker_bytes = end_marker.as_bytes();

    let mut cursor = find_subslice(body, marker_bytes).ok_or(MultipartError::InvalidBody)?;
    cursor += marker_bytes.len();

    let mut multipart = Multipart::default();

    loop {
        if body.get(cursor..cursor + 2) == Some(b"--") {
            break;
        }
        if body.get(cursor..cursor + 2) != Some(b"\r\n") {
            return Err(MultipartError::InvalidBody);
        }
        cursor += 2;

        let next =
            find_subslice(&body[cursor..], end_marker_bytes).ok_or(MultipartError::InvalidBody)?;
        let part = &body[cursor..cursor + next];
        cursor += next + end_marker_bytes.len();

        parse_part(part, &mut multipart)?;
    }

    Ok(multipart)
}

fn extract_boundary(content_type: &str) -> Option<String> {
    let mut is_multipart = false;
    let mut boundary = None;

    for param in content_type.split(';') {
        let param = param.trim();
        if param.eq_ignore_ascii_case("multipart/form-data") {
            is_multipart = true;
            continue;
        }
        if let Some(rest) = param.split_once('=') {
            if rest.0.eq_ignore_ascii_case("boundary") {
                let value = rest.1.trim().trim_matches('"');
                if !value.is_empty() {
                    boundary = Some(value.to_string());
                }
            }
        }
    }

    if is_multipart { boundary } else { None }
}

fn parse_part(part: &[u8], multipart: &mut Multipart) -> Result<(), MultipartError> {
    let header_end = find_subslice(part, b"\r\n\r\n").ok_or(MultipartError::InvalidPartHeaders)?;
    let headers =
        std::str::from_utf8(&part[..header_end]).map_err(|_| MultipartError::InvalidUtf8)?;
    let data = &part[header_end + 4..];

    let mut name = None;
    let mut filename = None;
    let mut content_type = None;

    for line in headers.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        let Some((raw_name, raw_value)) = line.split_once(':') else {
            return Err(MultipartError::InvalidPartHeaders);
        };
        let header_name = raw_name.trim();
        let header_value = raw_value.trim();

        if header_name.eq_ignore_ascii_case("content-disposition") {
            name = disposition_param(header_value, "name");
            filename = disposition_param(header_value, "filename");
        } else if header_name.eq_ignore_ascii_case("content-type") {
            content_type = Some(header_value.to_string());
        }
    }

    let name = name.ok_or(MultipartError::MissingDispositionName)?;

    if filename.is_some() {
        multipart.files.push(MultipartFile {
            name,
            filename,
            content_type,
            data: data.to_vec(),
        });
    } else {
        let value = std::str::from_utf8(data)
            .map_err(|_| MultipartError::InvalidUtf8)?
            .to_string();
        multipart.fields.insert(name, value);
    }

    Ok(())
}

fn disposition_param(header_value: &str, key: &str) -> Option<String> {
    for part in header_value.split(';') {
        let part = part.trim();
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        if k.eq_ignore_ascii_case(key) {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}
