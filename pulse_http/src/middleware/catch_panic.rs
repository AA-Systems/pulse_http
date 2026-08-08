use std::panic::AssertUnwindSafe;

use futures::FutureExt;

use crate::{
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
    router::BoxFuture,
};

pub struct CatchPanic;

impl Middleware for CatchPanic {
    fn handle(&self, request: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            match AssertUnwindSafe(next.run(request)).catch_unwind().await {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("handler panicked; returning 500");
                    Response::internal_server_error()
                }
            }
        })
    }
}
