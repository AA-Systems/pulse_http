use pulse_http::{Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoBody {
    pub message: String,
}

pub async fn echo(req: Request) -> Response {
    match req.json::<EchoBody>() {
        Ok(body) => Response::json(200, body),
        Err(_) => Response::bad_request(),
    }
}
