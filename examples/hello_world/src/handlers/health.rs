use http::{request::Request, response::Response};

pub fn health(_req: Request) -> Response {
    Response::ok("ok")
}
