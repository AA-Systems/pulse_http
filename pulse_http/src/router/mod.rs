use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::{
    helpers::split_segments::split_segments,
    rate_limit::Limit,
    request::{Method, Request},
    response::Response,
};

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type Handler = Arc<dyn Fn(Request) -> BoxFuture<Response> + Send + Sync>;

#[derive(Default)]
struct Node {
    static_children: HashMap<String, Node>,
    param_child: Option<(String, Box<Node>)>,
    handlers: HashMap<Method, Handler>,
    limits: HashMap<Method, Limit>,
}

#[derive(Default)]
pub struct Router {
    root: Node,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Get, path, handler, None)
    }

    pub fn get_with_rate_limit<F, Fut>(
        self,
        path: &str,
        handler: F,
        requests_per_minute: f64,
    ) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(
            Method::Get,
            path,
            handler,
            Some(Limit::per_minute(requests_per_minute)),
        )
    }

    pub fn post<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Post, path, handler, None)
    }

    pub fn post_with_rate_limit<F, Fut>(
        self,
        path: &str,
        handler: F,
        requests_per_minute: f64,
    ) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(
            Method::Post,
            path,
            handler,
            Some(Limit::per_minute(requests_per_minute)),
        )
    }

    pub fn put<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Put, path, handler, None)
    }

    pub fn delete<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Delete, path, handler, None)
    }

    pub fn route<F, Fut>(
        mut self,
        method: Method,
        path: &str,
        handler: F,
        rate_limit: Option<Limit>,
    ) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |request| Box::pin(handler(request)));
        self.insert(method, path, handler, rate_limit);
        self
    }

    fn insert(&mut self, method: Method, path: &str, handler: Handler, rate_limit: Option<Limit>) {
        let segments = split_segments(path);
        let mut node = &mut self.root;

        for segment in segments {
            if let Some(name) = segment.strip_prefix(':') {
                if node.param_child.is_none() {
                    node.param_child = Some((name.to_string(), Box::new(Node::default())));
                } else if let Some((existing, _)) = &node.param_child {
                    assert!(
                        existing == name,
                        "conflicting param names at the same position: :{existing} vs :{name}"
                    );
                }
                node = node.param_child.as_mut().unwrap().1.as_mut();
            } else {
                node = node.static_children.entry(segment.to_string()).or_default();
            }
        }

        assert!(
            !node.handlers.contains_key(&method),
            "duplicate route: {} {}",
            method.as_str(),
            path
        );
        node.handlers.insert(method, handler);
        if let Some(limit) = rate_limit {
            node.limits.insert(method, limit);
        }
    }

    pub fn match_route(&self, request: &mut Request) -> Option<(Handler, Option<Limit>)> {
        let segments = split_segments(&request.path);
        let mut node = &self.root;
        let mut params = HashMap::new();

        for segment in segments {
            if let Some(child) = node.static_children.get(segment) {
                node = child;
                continue;
            }

            if let Some((name, child)) = &node.param_child {
                params.insert(name.clone(), segment.to_string());
                node = child;
                continue;
            }

            return None;
        }

        let handler = node.handlers.get(&request.method)?.clone();
        let limit = node.limits.get(&request.method).copied();
        request.params = params;
        Some((handler, limit))
    }
}
