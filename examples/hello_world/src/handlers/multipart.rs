use pulse_http::{Request, Response};
use serde::Serialize;

#[derive(Serialize)]
struct FileInfo {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    size: usize,
}

#[derive(Serialize)]
struct MultipartSummary {
    fields: std::collections::HashMap<String, String>,
    files: Vec<FileInfo>,
}

pub async fn multipart(req: Request) -> Response {
    match req.multipart() {
        Ok(form) => {
            let files = form
                .files
                .into_iter()
                .map(|file| FileInfo {
                    name: file.name,
                    filename: file.filename,
                    content_type: file.content_type,
                    size: file.data.len(),
                })
                .collect();
            Response::json(
                200,
                MultipartSummary {
                    fields: form.fields,
                    files,
                },
            )
        }
        Err(_) => Response::bad_request(),
    }
}
