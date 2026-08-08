//! Async HTTP/1.1 framework built on Tokio (no Hyper / Axum).
//!
//! # Quick start
//!
//! ```ignore
//! use pulse_http::{Request, Response, Router, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new().get("/health", |_req: Request| async {
//!         Response::ok("ok")
//!     });
//!
//!     Server::bind("127.0.0.1:3000".into())
//!         .await
//!         .router(router)
//!         .serve()
//!         .await;
//! }
//! ```
//!
//! # v0.1 scope
//!
//! Included: keep-alive, `Content-Length` bodies, routing, middleware, JSON,
//! url-encoded forms, typed state, rate limits, CORS, graceful shutdown.
//!
//! Not yet (planned for later): chunked transfer encoding, multipart uploads,
//! streaming responses, write timeouts, fuzz tests, and benchmarks.

pub mod constants;
pub mod errors;
mod headers;
mod helpers;
pub mod middleware;
pub mod rate_limit;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod state;

pub use errors::FormError;
pub use middleware::{CatchPanic, Cors, Middleware, RequestId, RequestLogger, SecurityHeaders};
pub use rate_limit::{Limit, RateLimit};
pub use request::{Method, Request};
pub use response::Response;
pub use router::{Handler, Router};
pub use server::Server;
pub use state::State;
