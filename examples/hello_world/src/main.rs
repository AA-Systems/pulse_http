mod handlers;

use handlers::{health::health, hello::hello};
use http::{router::Router, server::Server};
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/health", health)
        .get("/hello/:name", hello);

    Server::bind(String::from("127.0.0.1:3001"))
        .await
        .router(router)
        .serve()
        .await;

    Ok(())
}
