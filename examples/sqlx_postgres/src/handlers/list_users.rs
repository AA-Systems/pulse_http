use pulse_http::{Request, Response};
use serde::Serialize;
use sqlx::{Pool, Postgres};

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserRow {
    id: i32,
    name: String,
    email: String,
}

pub async fn list_users(req: Request) -> Response {
    let Some(pg_pool) = req.state.get::<Pool<Postgres>>() else {
        return Response::internal_server_error();
    };

    match sqlx::query_as::<_, UserRow>("SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(pg_pool.as_ref())
        .await
    {
        Ok(users) => Response::json(200, users),
        Err(_) => Response::internal_server_error(),
    }
}
