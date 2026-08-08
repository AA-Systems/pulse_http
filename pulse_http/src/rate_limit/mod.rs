use crate::{helpers::rate_limit_key::rate_limit_key, request::Request, router::BoxFuture};
use futures::lock::Mutex;
use std::{collections::HashMap, sync::Arc, time::Instant};

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct Limit {
    pub capacity: f64,
    pub refill_per_sec: f64,
}

impl Limit {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            refill_per_sec,
        }
    }

    pub fn per_minute(requests: f64) -> Self {
        Self::new(requests, requests / 60.0)
    }
}

pub struct RateLimit {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    default_limit: Limit,
}

impl RateLimit {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            default_limit: Limit::per_minute(10.0),
        }
    }

    pub fn with_default(default_limit: Limit) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            default_limit,
        }
    }

    pub fn default_limit(&self) -> Limit {
        self.default_limit
    }

    pub fn allow_request(&self, request: &Request, limit: Limit) -> BoxFuture<bool> {
        let key = rate_limit_key(request.ip_addr, request.method.as_str(), &request.path);
        self.allow(key, limit)
    }

    pub fn allow(&self, key: String, limit: Limit) -> BoxFuture<bool> {
        let buckets = Arc::clone(&self.buckets);

        Box::pin(async move {
            let mut buckets = buckets.lock().await;
            let now = Instant::now();

            let bucket = buckets.entry(key).or_insert_with(|| Bucket {
                tokens: limit.capacity,
                last_refill: now,
            });

            let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
            bucket.tokens = (bucket.tokens + elapsed * limit.refill_per_sec).min(limit.capacity);
            bucket.last_refill = now;

            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                true
            } else {
                false
            }
        })
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self::new()
    }
}
