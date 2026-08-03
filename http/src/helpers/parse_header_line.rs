use crate::errors::ParseError;

pub fn parse_header_line(line: &str) -> Result<(String, String), ParseError> {
    let (name, value) = line.split_once(':').ok_or(ParseError::InvalidHeaderLine)?;
    let name = name.trim();
    if name.is_empty() || name.contains(' ') {
        return Err(ParseError::EmptyHeaderName);
    }
    Ok((name.to_ascii_lowercase(), value.trim().to_string()))
}
