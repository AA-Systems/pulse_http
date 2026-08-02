use http::server::Server;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    Server::bind(String::from("127.0.0.1:3001"))
        .await
        .serve()
        .await;

    Ok(())
}
