mod handlers;

use handlers::{echo::echo, form::form, health::health, hello::hello};
use pulse_http::{Router, Server};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .get("/health", health)
        .get("/hello/:name", hello)
        .post("/echo", echo)
        .post("/form", form);

    Server::bind(String::from("127.0.0.1:3000"))
        .await
        .router(router)
        .serve()
        .await;
}
