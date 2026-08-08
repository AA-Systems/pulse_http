mod handlers;

use handlers::{health::health, insert_user::insert_user, list_users::list_users};
use pulse_http::{Router, Server};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://myuser:mysecretpassword@localhost:5432/mydatabase".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to postgres");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        )",
    )
    .execute(&pool)
    .await
    .expect("failed to ensure users table");

    let router = Router::new()
        .get("/health", health)
        .get("/users", list_users)
        .post("/users", insert_user);

    println!("sqlx_postgres listening on http://127.0.0.1:3001");
    Server::bind(String::from("127.0.0.1:3001"))
        .await
        .state(pool)
        .router(router)
        .serve()
        .await;
}
