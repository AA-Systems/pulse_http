use pulse_http::{Request, Response};
use std::path::PathBuf;

pub async fn download(_req: Request) -> Response {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    match Response::file(path) {
        Ok(response) => response,
        Err(_) => Response::not_found(),
    }
}
