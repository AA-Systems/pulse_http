use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{
    middleware::{Middleware, Next},
    request::Request,
    response::Response,
};

pub struct CatchPanic;

impl Middleware for CatchPanic {
    fn handle(&self, request: Request, next: Next) -> Response {
        match catch_unwind(AssertUnwindSafe(|| next.run(request))) {
            Ok(response) => response,
            Err(_) => {
                eprintln!("handler panicked; returning 500");
                Response::internal_server_error()
            }
        }
    }
}
