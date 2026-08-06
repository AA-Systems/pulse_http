use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
    router::BoxFuture,
};

const REQUEST_ID_HEADER: &str = "x-request-id";

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct RequestId;

impl Middleware for RequestId {
    fn handle(&self, mut request: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            let request_id = request
                .headers
                .get(REQUEST_ID_HEADER)
                .cloned()
                .unwrap_or_else(generate_request_id);

            request
                .headers
                .insert(REQUEST_ID_HEADER.to_string(), request_id.clone());

            let mut response = next.run(request).await;
            response
                .headers
                .insert(REQUEST_ID_HEADER.to_string(), request_id);
            response
        })
    }
}

fn generate_request_id() -> String {
    let n = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{millis}-{n}")
}
