use std::time::Instant;

use crate::{
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
};

pub struct RequestLogger;

impl Middleware for RequestLogger {
    fn handle(&self, request: Request, next: Next) -> Response {
        let method = request.method.as_str().to_string();
        let path = request.path.clone();
        let request_id = request
            .headers
            .get("x-request-id")
            .cloned()
            .unwrap_or_else(|| "-".into());
        let started = Instant::now();

        let response = next.run(request);

        println!(
            "[{request_id}] {method} {path} -> {} ({} ms)",
            response.status,
            started.elapsed().as_millis()
        );

        response
    }
}
