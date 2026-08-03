use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    headers::Headers, helpers::find_subslice::find_subslice, requests::bad_request::bad_request,
};

const MAX_HEADER_SIZE: usize = 32 * 1024;
const HEADER_END: &[u8] = b"\r\n\r\n";

pub async fn handle_client(mut stream: TcpStream) {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        match stream.read(&mut chunk).await {
            Ok(0) => return,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);

                if buffer.len() > MAX_HEADER_SIZE {
                    bad_request(&mut stream).await;
                    return;
                }

                if let Some(end) = find_subslice(&buffer, HEADER_END) {
                    let header_bytes = &buffer[..end + HEADER_END.len()];

                    let headers = match Headers::parse(header_bytes) {
                        Ok(headers) => headers,
                        Err(_) => {
                            bad_request(&mut stream).await;
                            return;
                        }
                    };

                    println!("{:#?}", headers);

                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nok",
                        )
                        .await;
                    return;
                }
            }
            Err(_) => return,
        }
    }
}
