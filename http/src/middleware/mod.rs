use crate::{
    request::Request,
    response::Response,
    router::{BoxFuture, Handler},
};
use std::sync::Arc;

pub mod catch_panic;
pub mod logger;
pub mod request_id;

pub use catch_panic::CatchPanic;
pub use logger::RequestLogger;
pub use request_id::RequestId;

pub trait Middleware: Send + Sync {
    fn handle(&self, request: Request, next: Next) -> BoxFuture<Response>;
}

pub struct Next {
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    index: usize,
    endpoint: Handler,
}

impl Next {
    pub fn new(middlewares: Arc<Vec<Arc<dyn Middleware>>>, endpoint: Handler) -> Self {
        Self {
            middlewares,
            index: 0,
            endpoint,
        }
    }

    pub fn run(self, request: Request) -> BoxFuture<Response> {
        if self.index < self.middlewares.len() {
            let middleware = Arc::clone(&self.middlewares[self.index]);
            let next = Next {
                middlewares: self.middlewares,
                index: self.index + 1,
                endpoint: self.endpoint,
            };
            middleware.handle(request, next)
        } else {
            (self.endpoint)(request)
        }
    }
}
