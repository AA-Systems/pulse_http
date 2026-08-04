use crate::{
    constants::{MAX_BODY_SIZE, MAX_REQUESTS_PER_CONNECTION},
    errors::ReadHeadersError,
    helpers::{
        read_body::read_content_length_body, read_headers::read_headers,
        should_keep_alive::should_keep_alive,
    },
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
    router::Router,
    state::State,
};
use std::sync::Arc;
use tokio::net::TcpStream;

pub async fn handle_client(
    mut stream: TcpStream,
    router: Arc<Router>,
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    state: Arc<State>,
) {
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
                Response::bad_request().write_to_stream(&mut stream).await;
                return;
            }
        };

        if headers.fields.contains_key("transfer-encoding") {
            Response::bad_request().write_to_stream(&mut stream).await;
            return;
        }

        let content_length = match headers.fields.get("content-length") {
            Some(value) => match value.parse::<usize>() {
                Ok(n) => n,
                Err(_) => {
                    Response::bad_request().write_to_stream(&mut stream).await;
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
                Response::bad_request().write_to_stream(&mut stream).await;
                return;
            }
        };

        let request = match Request::from_headers(headers, body, (*state).clone()) {
            Some(request) => request,
            None => {
                Response::bad_request().write_to_stream(&mut stream).await;
                return;
            }
        };

        let keep_alive = should_keep_alive(&request);

        let router = Arc::clone(&router);
        let endpoint = Arc::new(move |mut req: Request| match router.match_route(&mut req) {
            Some(handler) => handler(req),
            None => Response::not_found(),
        });

        let response = Next::new(Arc::clone(&middlewares), endpoint).run(request);

        response.write_to_stream(&mut stream).await;
        request_count += 1;
        buffer = leftover;

        if !keep_alive {
            return;
        }
    }
}
