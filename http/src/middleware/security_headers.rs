use crate::{
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
    router::BoxFuture,
};

pub struct SecurityHeaders;

impl Middleware for SecurityHeaders {
    fn handle(&self, request: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            let mut response = next.run(request).await;
            response.apply_security_headers();
            response
        })
    }
}
