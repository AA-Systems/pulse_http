use crate::helpers::read_with_timeout::read_with_timeout;
use tokio::net::TcpStream;

pub async fn read_content_length_body(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    body_start: usize,
    content_length: usize,
    max_body_size: usize,
) -> Result<(Vec<u8>, Vec<u8>), ()> {
    if content_length > max_body_size {
        return Err(());
    }

    let mut chunk = [0u8; 1024];

    while buffer.len().saturating_sub(body_start) < content_length {
        match read_with_timeout(stream, &mut chunk).await {
            Ok(0) => return Err(()),
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(()) => return Err(()),
        }
    }

    let body_end = body_start + content_length;
    Ok((
        buffer[body_start..body_end].to_vec(),
        buffer[body_end..].to_vec(),
    ))
}
