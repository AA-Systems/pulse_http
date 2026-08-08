use crate::{
    constants::{HEADER_END, MAX_HEADER_SIZE},
    errors::ReadHeadersError,
    headers::Headers,
    helpers::{find_subslice::find_subslice, read_with_timeout::read_with_timeout},
};
use tokio::net::TcpStream;

pub async fn read_headers(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    read_timeout_sec: u64,
) -> Result<(Headers, usize), ReadHeadersError> {
    let mut chunk = [0u8; 1024];

    loop {
        if let Some(end) = find_subslice(buffer, HEADER_END) {
            let header_end = end + HEADER_END.len();
            if header_end > MAX_HEADER_SIZE {
                return Err(ReadHeadersError::BadRequest);
            }

            let headers =
                Headers::parse(&buffer[..header_end]).map_err(|_| ReadHeadersError::BadRequest)?;
            return Ok((headers, header_end));
        }

        if buffer.len() > MAX_HEADER_SIZE {
            return Err(ReadHeadersError::BadRequest);
        }

        match read_with_timeout(stream, &mut chunk, read_timeout_sec).await {
            Ok(0) => {
                return if buffer.is_empty() {
                    Err(ReadHeadersError::Closed)
                } else {
                    Err(ReadHeadersError::BadRequest)
                };
            }
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(()) => return Err(ReadHeadersError::Closed),
        }
    }
}
