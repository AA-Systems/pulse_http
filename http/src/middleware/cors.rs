use crate::{
    middleware::{Middleware, Next},
    request::{Method, Request},
    response::Response,
    router::BoxFuture,
};

pub struct Cors {
    pub origins: Vec<String>,
    pub allowed_methods: String,
    pub allowed_headers: String,
}

impl Cors {
    pub fn new() -> Self {
        Self {
            origins: vec!["http://localhost:3000".into()],
            allowed_methods: "GET, POST, PUT, DELETE, HEAD, OPTIONS".into(),
            allowed_headers: "content-type, x-request-id".into(),
        }
    }

    pub fn origins(mut self, origins: Vec<String>) -> Self {
        self.origins = origins;
        self
    }

    pub fn allowed_methods(mut self, methods: impl Into<String>) -> Self {
        self.allowed_methods = methods.into();
        self
    }

    pub fn allowed_headers(mut self, headers: impl Into<String>) -> Self {
        self.allowed_headers = headers.into();
        self
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.origins.iter().any(|allowed| allowed == origin)
    }

    fn apply_cors_headers(&self, response: &mut Response, origin: &str) {
        response
            .headers
            .insert("access-control-allow-origin".into(), origin.to_string());
        response.headers.insert(
            "access-control-allow-methods".into(),
            self.allowed_methods.clone(),
        );
        response.headers.insert(
            "access-control-allow-headers".into(),
            self.allowed_headers.clone(),
        );
        response.headers.insert("vary".into(), "Origin".into());
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Cors {
    fn handle(&self, request: Request, next: Next) -> BoxFuture<Response> {
        let origins = self.origins.clone();
        let allowed_methods = self.allowed_methods.clone();
        let allowed_headers = self.allowed_headers.clone();

        Box::pin(async move {
            let cors = Cors {
                origins,
                allowed_methods,
                allowed_headers,
            };

            let origin = request.headers.get("origin").cloned();

            if request.method == Method::Options {
                let mut response = Response::no_content();
                if let Some(origin) = origin.as_deref() {
                    if cors.is_origin_allowed(origin) {
                        cors.apply_cors_headers(&mut response, origin);
                        response
                            .headers
                            .insert("access-control-max-age".into(), "86400".into());
                    }
                }
                return response;
            }

            let mut response = next.run(request).await;

            if let Some(origin) = origin.as_deref() {
                if cors.is_origin_allowed(origin) {
                    cors.apply_cors_headers(&mut response, origin);
                }
            }

            response
        })
    }
}
