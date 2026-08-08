mod handlers;

use crate::handlers::insert_user::insert_user;
use handlers::{echo::echo, form::form, health::health, hello::hello};
use http::{router::Router, server::Server};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let database_url = "postgres://myuser:mysecretpassword@localhost:5432/mydatabase";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("failed to connect to postgres");

    let router = Router::new()
        .get_with_rate_limit("/health", health, 5.0)
        .get("/hello/:name", hello)
        .post("/echo", echo)
        .post("/form", form)
        .post("/insert_user", insert_user);

    Server::bind(String::from("127.0.0.1:3001"))
        .await
        .state(pool)
        .router(router)
        .serve()
        .await;
}
