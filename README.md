# PulseHTTP

Async HTTP/1.1 framework for Rust, built on [Tokio](https://tokio.rs) from raw TCP sockets

[![Crates.io](https://img.shields.io/crates/v/pulse_http.svg)](https://crates.io/crates/pulse_http)
[![Docs.rs](https://docs.rs/pulse_http/badge.svg)](https://docs.rs/pulse_http)

## Install

```toml
[dependencies]
pulse_http = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick start

```rust
use pulse_http::{Request, Response, Router, Server};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .get("/health", |_req: Request| async { Response::ok("ok") })
        .get("/hello/:name", |req: Request| async move {
            let name = req
                .params
                .get("name")
                .cloned()
                .unwrap_or_else(|| "world".into());
            Response::ok(format!("hello {name}"))
        });

    Server::bind("127.0.0.1:3000".into())
        .await
        .router(router)
        .serve()
        .await;
}
```

Ctrl+C triggers graceful shutdown (stop accept → drain connections → grace abort).

## Features

- Tokio TCP listener, task-per-connection, admission semaphore
- HTTP/1.1 keep-alive, `Content-Length` and chunked request bodies
- Read and write timeouts (slow clients close the connection)
- Trie router (static + `:param`), query params
- Async middleware (request id, logging, CORS, security headers, panic catch)
- JSON + `application/x-www-form-urlencoded` + `multipart/form-data`
- Typed app `State`
- Per-route token-bucket rate limits
- Configurable max connections, max body size, read/write timeouts
- Graceful shutdown

## Examples

From this repo:

```bash
# routing, JSON, forms — no database
cargo run -p hello_world

# per-route rate limiting (429 after burst)
cargo run -p rate_limit

# typed State + Postgres (sqlx)
docker compose -f examples/docker-compose.yml up -d
cargo run -p sqlx_postgres
```

| Example         | Port | Focus        |
| --------------- | ---- | ------------ |
| `hello_world`   | 3000 | Basics       |
| `sqlx_postgres` | 3001 | `State` + DB |
| `rate_limit`    | 3002 | Route limits |

## Builder knobs

```rust
Server::bind("127.0.0.1:3000".into())
    .await
    .max_connects(256)
    .max_body_size(5 * 1024 * 1024)
    .read_timeout_sec(60)
    .write_timeout_sec(60)
    .router(router)
    .serve()
    .await;
```

## Not yet (later)

Streaming responses, fuzz tests, benchmarks.

## License

MIT
