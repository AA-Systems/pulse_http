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
//! # Current scope
//!
//! Included: keep-alive, `Content-Length` and chunked request bodies, routing,
//! middleware, JSON, url-encoded forms, multipart form-data, streaming file
//! responses, typed state, rate limits, CORS, read/write timeouts, graceful
//! shutdown.
//!
//! Not yet: fuzz tests and benchmarks.

pub mod body;
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

pub use body::Body;
pub use errors::{FormError, MultipartError};
pub use helpers::parse_multipart::{Multipart, MultipartFile};
pub use middleware::{CatchPanic, Cors, Middleware, RequestId, RequestLogger, SecurityHeaders};
pub use rate_limit::{Limit, RateLimit};
pub use request::{Method, Request};
pub use response::Response;
pub use router::{Handler, Router};
pub use server::Server;
pub use state::State;
