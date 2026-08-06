use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::{
    helpers::split_segments::split_segments,
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
        self.route(Method::Get, path, handler)
    }

    pub fn post<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Post, path, handler)
    }

    pub fn put<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Put, path, handler)
    }

    pub fn delete<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.route(Method::Delete, path, handler)
    }

    pub fn route<F, Fut>(mut self, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |request| Box::pin(handler(request)));
        self.insert(method, path, handler);
        self
    }

    fn insert(&mut self, method: Method, path: &str, handler: Handler) {
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
    }

    pub fn match_route(&self, request: &mut Request) -> Option<Handler> {
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
        request.params = params;
        Some(handler)
    }
}
