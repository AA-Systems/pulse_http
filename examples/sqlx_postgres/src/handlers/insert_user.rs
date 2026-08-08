use pulse_http::{Request, Response};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    name: String,
    email: String,
}

pub async fn insert_user(req: Request) -> Response {
    let user = match req.json::<User>() {
        Ok(user) => user,
        Err(_) => return Response::bad_request(),
    };

    let Some(pg_pool) = req.state.get::<Pool<Postgres>>() else {
        return Response::internal_server_error();
    };

    match sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind(&user.name)
        .bind(&user.email)
        .execute(pg_pool.as_ref())
        .await
    {
        Ok(_) => Response::json(201, user),
        Err(_) => Response::internal_server_error(),
    }
}
