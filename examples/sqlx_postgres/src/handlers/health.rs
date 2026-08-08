use pulse_http::{Request, Response};

pub async fn health(_req: Request) -> Response {
    Response::ok("ok")
}
