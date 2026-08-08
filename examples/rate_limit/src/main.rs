use pulse_http::{Request, Response, Router, Server};

async fn health(_req: Request) -> Response {
    Response::ok("ok")
}

async fn limited(_req: Request) -> Response {
    Response::ok("allowed")
}

#[tokio::main]
async fn main() {
    // /limited allows 3 requests per minute per IP (then 429).
    let router = Router::new()
        .get("/health", health)
        .get_with_rate_limit("/limited", limited, 3.0);

    println!("rate_limit listening on http://127.0.0.1:3002");
    Server::bind(String::from("127.0.0.1:3002"))
        .await
        .router(router)
        .serve()
        .await;
}
