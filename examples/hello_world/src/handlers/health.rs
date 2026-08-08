use pulse_http::{request::Request, response::Response};

pub async fn health(_req: Request) -> Response {
    Response::ok("ok")
}
