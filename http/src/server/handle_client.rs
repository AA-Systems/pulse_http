use tokio::net::TcpStream;

use crate::{
    constants::{MAX_BODY_SIZE, MAX_REQUESTS_PER_CONNECTION},
    errors::ReadHeadersError,
    helpers::{
        read_body::read_content_length_body, read_headers::read_headers,
        should_keep_alive::should_keep_alive,
    },
    response::{bad_request_response::bad_request_response, ok_response::ok_response},
};

pub async fn handle_client(mut stream: TcpStream) {
    let mut buffer = Vec::new();
    let mut request_count = 0u32;

    loop {
        if request_count >= MAX_REQUESTS_PER_CONNECTION {
            return;
        }

        let (headers, body_start) = match read_headers(&mut stream, &mut buffer).await {
            Ok(parsed) => parsed,
            Err(ReadHeadersError::Closed) => return,
            Err(ReadHeadersError::BadRequest) => {
                bad_request_response(&mut stream).await;
                return;
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

        let (body, leftover) = match read_content_length_body(
            &mut stream,
            &mut buffer,
            body_start,
            content_length,
            MAX_BODY_SIZE,
        )
        .await
        {
            Ok(result) => result,
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
        request_count += 1;

        buffer = leftover;

        if !should_keep_alive(&headers) {
            return;
        }
    }
}
