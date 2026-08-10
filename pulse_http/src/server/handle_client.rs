use crate::{
    constants::MAX_REQUESTS_PER_CONNECTION,
    errors::ReadHeadersError,
    helpers::{
        read_body::read_content_length_body,
        read_chunked_body::{is_chunked_transfer_encoding, read_chunked_body},
        read_headers::read_headers,
        should_keep_alive::should_keep_alive,
    },
    middleware::{Middleware, Next},
    rate_limit::RateLimit,
    request::Request,
    response::Response,
    router::Router,
    state::State,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::watch};

pub async fn handle_client(
    mut stream: TcpStream,
    router: Arc<Router>,
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    state: Arc<State>,
    rate_limit: Arc<RateLimit>,
    peer_addr: SocketAddr,
    mut shutdown_rx: watch::Receiver<bool>,
    max_body_size: usize,
    read_timeout_sec: u64,
    write_timeout_sec: u64,
) {
    let mut buffer = Vec::new();
    let mut request_count = 0u32;

    loop {
        if request_count >= MAX_REQUESTS_PER_CONNECTION {
            return;
        }

        if *shutdown_rx.borrow() {
            return;
        }

        let (headers, body_start) = tokio::select! {
            biased;
            result = shutdown_rx.changed() => {
                match result {
                    Ok(()) if *shutdown_rx.borrow() => return,
                    Ok(()) => continue,
                    Err(_) => return,
                }
            }
            result = read_headers(&mut stream, &mut buffer, read_timeout_sec) => {
                match result {
                    Ok(parsed) => parsed,
                    Err(ReadHeadersError::Closed) => return,
                    Err(ReadHeadersError::BadRequest) => {
                        let _ = Response::bad_request()
                            .write_to_stream(write_timeout_sec, &mut stream)
                            .await;
                        return;
                    }
                }
            }
        };

        let (body, leftover) = if let Some(te) = headers.fields.get("transfer-encoding") {
            if !is_chunked_transfer_encoding(te) {
                let _ = Response::bad_request()
                    .write_to_stream(write_timeout_sec, &mut stream)
                    .await;
                return;
            }

            match read_chunked_body(
                &mut stream,
                &mut buffer,
                body_start,
                max_body_size,
                read_timeout_sec,
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    let _ = Response::bad_request()
                        .write_to_stream(write_timeout_sec, &mut stream)
                        .await;
                    return;
                }
            }
        } else {
            let content_length = match headers.fields.get("content-length") {
                Some(value) => match value.parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => {
                        let _ = Response::bad_request()
                            .write_to_stream(write_timeout_sec, &mut stream)
                            .await;
                        return;
                    }
                },
                None => 0,
            };

            match read_content_length_body(
                &mut stream,
                &mut buffer,
                body_start,
                content_length,
                max_body_size,
                read_timeout_sec,
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    let _ = Response::bad_request()
                        .write_to_stream(write_timeout_sec, &mut stream)
                        .await;
                    return;
                }
            }
        };

        let request = match Request::from_headers(headers, body, (*state).clone(), peer_addr.ip()) {
            Some(request) => request,
            None => {
                let _ = Response::bad_request()
                    .write_to_stream(write_timeout_sec, &mut stream)
                    .await;
                return;
            }
        };

        let keep_alive = should_keep_alive(&request);

        let router = Arc::clone(&router);
        let rate_limit = Arc::clone(&rate_limit);
        let endpoint = Arc::new(
            move |mut req: Request| -> crate::router::BoxFuture<Response> {
                match router.match_route(&mut req) {
                    Some((handler, route_limit)) => {
                        let rate_limit = Arc::clone(&rate_limit);
                        Box::pin(async move {
                            let limit = route_limit.unwrap_or_else(|| rate_limit.default_limit());
                            if !rate_limit.allow_request(&req, limit).await {
                                return Response::too_many_requests();
                            }
                            handler(req).await
                        })
                    }
                    None => Box::pin(async move { Response::not_found() }),
                }
            },
        );

        let response = Next::new(Arc::clone(&middlewares), endpoint)
            .run(request)
            .await;

        if response
            .write_to_stream(write_timeout_sec, &mut stream)
            .await
            .is_err()
        {
            return;
        }

        request_count += 1;
        buffer = leftover;

        if !keep_alive || *shutdown_rx.borrow() {
            return;
        }
    }
}
