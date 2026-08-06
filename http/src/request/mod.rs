use crate::{headers::Headers, helpers::split_path_and_query::split_path_and_query, state::State};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
}

impl Method {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "HEAD" => Some(Self::Head),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: Vec<u8>,
    pub http_version: String,
    pub state: State,
}

impl Request {
    pub fn from_headers(headers: Headers, body: Vec<u8>, state: State) -> Option<Self> {
        let method = Method::parse(&headers.method)?;
        let (path, query) = split_path_and_query(&headers.path);

        Some(Self {
            method,
            path,
            query,
            headers: headers.fields,
            params: HashMap::new(),
            body,
            http_version: headers.http_version,
            state,
        })
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}
