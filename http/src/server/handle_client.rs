use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    headers::Headers,
    helpers::{find_subslice::find_subslice, read_body::read_content_length_body},
    response::{bad_request_response::bad_request_response, ok_response::ok_response},
};

const MAX_HEADER_SIZE: usize = 32 * 1024;
const MAX_BODY_SIZE: usize = 1024 * 1024;
const HEADER_END: &[u8] = b"\r\n\r\n";

pub async fn handle_client(mut stream: TcpStream) {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    let (headers, body_start) = loop {
        match stream.read(&mut chunk).await {
            Ok(0) => return,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);

                if buffer.len() > MAX_HEADER_SIZE {
                    if find_subslice(&buffer, HEADER_END).is_none() {
                        bad_request_response(&mut stream).await;
                        return;
                    }
                }

                if let Some(end) = find_subslice(&buffer, HEADER_END) {
                    let header_end = end + HEADER_END.len();
                    if header_end > MAX_HEADER_SIZE {
                        bad_request_response(&mut stream).await;
                        return;
                    }

                    let headers = match Headers::parse(&buffer[..header_end]) {
                        Ok(headers) => headers,
                        Err(_) => {
                            bad_request_response(&mut stream).await;
                            return;
                        }
                    };

                    break (headers, header_end);
                }
            }
            Err(_) => return,
        }
    };

    println!("{:#?}", headers);

    if headers.fields.contains_key("transfer-encoding") {
        bad_request_response(&mut stream).await;
        return;
    }

    let content_length = match headers.fields.get("content-length") {
        Some(value) => match value.parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                bad_request_response(&mut stream).await;
                return;
            }
        },
        None => 0,
    };

    let body = match read_content_length_body(
        &mut stream,
        &mut buffer,
        body_start,
        content_length,
        MAX_BODY_SIZE,
    )
    .await
    {
        Ok(body) => body,
        Err(_) => {
            bad_request_response(&mut stream).await;
            return;
        }
    };

    println!(
        "body ({} bytes): {:?}",
        body.len(),
        String::from_utf8_lossy(&body)
    );

    ok_response(&mut stream).await;
}
