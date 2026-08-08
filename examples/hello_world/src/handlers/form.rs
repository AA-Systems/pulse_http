use pulse_http::{request::Request, response::Response};

pub async fn form(req: Request) -> Response {
    match req.form() {
        Ok(fields) => Response::json(200, fields),
        Err(_) => Response::bad_request(),
    }
}
