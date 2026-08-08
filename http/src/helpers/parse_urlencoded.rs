use std::collections::HashMap;

use crate::errors::FormError;

/// Parses `application/x-www-form-urlencoded` text into key/value pairs.
/// `+` becomes space; `%XX` is percent-decoded.
pub fn parse_urlencoded(input: &str) -> Result<HashMap<String, String>, FormError> {
    let mut form = HashMap::new();

    if input.is_empty() {
        return Ok(form);
    }

    for pair in input.split('&') {
        if pair.is_empty() {
            continue;
        }

        let (raw_key, raw_value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };

        let key = percent_decode(raw_key)?;
        let value = percent_decode(raw_value)?;
        form.insert(key, value);
    }

    Ok(form)
}

fn percent_decode(input: &str) -> Result<String, FormError> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                output.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(FormError::InvalidPercentEncoding);
                }
                let hi = from_hex(bytes[i + 1])?;
                let lo = from_hex(bytes[i + 2])?;
                output.push((hi << 4) | lo);
                i += 3;
            }
            byte => {
                output.push(byte);
                i += 1;
            }
        }
    }

    String::from_utf8(output).map_err(|_| FormError::InvalidUtf8)
}

fn from_hex(byte: u8) -> Result<u8, FormError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(FormError::InvalidPercentEncoding),
    }
}
