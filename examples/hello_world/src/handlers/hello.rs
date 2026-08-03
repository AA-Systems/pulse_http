use http::{request::Request, response::Response};

pub fn hello(req: Request) -> Response {
    let name = req
        .params
        .get("name")
        .cloned()
        .unwrap_or_else(|| "world".into());
    Response::ok(format!("hello {name}"))
}
