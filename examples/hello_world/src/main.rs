mod handlers;

use handlers::{
    download::download, echo::echo, form::form, health::health, hello::hello, multipart::multipart,
};
use pulse_http::{Router, Server};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .get("/health", health)
        .get("/hello/:name", hello)
        .get("/download", download)
        .post("/echo", echo)
        .post("/form", form)
        .post("/multipart", multipart);

    Server::bind(String::from("127.0.0.1:3000"))
        .await
        .router(router)
        .serve()
        .await;
}
