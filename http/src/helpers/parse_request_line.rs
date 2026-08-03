use crate::errors::ParseError;

pub fn parse_request_line(line: &str) -> Result<(String, String, String), ParseError> {
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(ParseError::InvalidRequestLine)?;
    let path = parts.next().ok_or(ParseError::InvalidRequestLine)?;
    let http_version = parts.next().ok_or(ParseError::InvalidRequestLine)?;

    if parts.next().is_some() {
        return Err(ParseError::InvalidRequestLine);
    }

    if method != "GET"
        && method != "POST"
        && method != "DELETE"
        && method != "PUT"
        && method != "HEAD"
    {
        return Err(ParseError::UnsupportedMethod);
    }

    if !http_version.starts_with("HTTP/") {
        return Err(ParseError::InvalidRequestLine);
    }

    Ok((
        method.to_string(),
        path.to_string(),
        http_version.to_string(),
    ))
}
