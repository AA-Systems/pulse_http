use crate::{
    constants::{
        DEFAULT_MAX_BODY_SIZE, DEFAULT_MAX_CONNECTIONS, DEFAULT_READ_TIMEOUT_SECS,
        SHUTDOWN_GRACE_SECS,
    },
    middleware::{CatchPanic, Cors, Middleware, RequestId, RequestLogger, SecurityHeaders},
    rate_limit::RateLimit,
    router::Router,
    server::handle_client::handle_client,
    state::State,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    signal,
    sync::{Semaphore, watch},
    task::JoinSet,
    time::sleep,
};

pub mod handle_client;

pub struct Server {
    pub listener: TcpListener,
    pub max_connections: u16,
    pub max_body_size: usize,
    pub read_timeout_sec: u64,
    pub router: Router,
    pub middlewares: Vec<Arc<dyn Middleware>>,
    pub state: State,
    pub rate_limit: Arc<RateLimit>,
}

impl Server {
    pub async fn bind(address: String) -> Self {
        Self {
            listener: Result::expect(
                TcpListener::bind(address).await,
                "Unable to initialize tcp listener",
            ),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            max_body_size: DEFAULT_MAX_BODY_SIZE,
            read_timeout_sec: DEFAULT_READ_TIMEOUT_SECS,
            router: Router::new(),
            middlewares: vec![
                Arc::new(RequestId),
                Arc::new(RequestLogger),
                Arc::new(Cors::new()),
                Arc::new(SecurityHeaders),
                Arc::new(CatchPanic),
            ],
            state: State::new(),
            rate_limit: Arc::new(RateLimit::new()),
        }
    }

    pub fn max_connects(self, connections: u16) -> Self {
        Self {
            max_connections: connections,
            ..self
        }
    }

    pub fn max_body_size(self, body_size: usize) -> Self {
        Self {
            max_body_size: body_size,
            ..self
        }
    }

    pub fn read_timeout_sec(self, read_timeout: u64) -> Self {
        Self {
            read_timeout_sec: read_timeout,
            ..self
        }
    }

    pub fn router(self, router: Router) -> Self {
        Self { router, ..self }
    }

    pub fn middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    pub fn rate_limit(self, rate_limit: RateLimit) -> Self {
        Self {
            rate_limit: Arc::new(rate_limit),
            ..self
        }
    }

    pub fn state<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        let temp_state = self.state.insert(value);
        self.state = temp_state;
        self
    }

    pub async fn serve(self) {
        println!("Server started");

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let admission = Arc::new(Semaphore::new(self.max_connections as usize));
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let state = Arc::new(self.state);
        let rate_limit = self.rate_limit;
        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    println!("Shutdown signal received, draining connections...");
                    let _ = shutdown_tx.send(true);
                    break;
                }
                stream_result = self.listener.accept() => {
                    match stream_result {
                        Ok((stream, peer_addr)) => {
                            let permit = match admission.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_error) => {
                                    drop(stream);
                                    continue;
                                }
                            };
                            let router = Arc::clone(&router);
                            let middlewares = Arc::clone(&middlewares);
                            let state = Arc::clone(&state);
                            let rate_limit = Arc::clone(&rate_limit);
                            let shutdown_rx = shutdown_rx.clone();
                            tasks.spawn(async move {
                                let _permit = permit;
                                handle_client(
                                    stream,
                                    router,
                                    middlewares,
                                    state,
                                    rate_limit,
                                    peer_addr,
                                    shutdown_rx,
                                    self.max_body_size,
                                    self.read_timeout_sec
                                )
                                .await;
                            });
                        }
                        Err(_error) => {}
                    }
                }
            }
        }

        drop(self.listener);

        tokio::select! {
            _ = async {
                while tasks.join_next().await.is_some() {}
            } => {
                println!("All connections drained");
            }
            _ = sleep(Duration::from_secs(SHUTDOWN_GRACE_SECS)) => {
                println!(
                    "Grace period ({}s) elapsed, aborting remaining connections",
                    SHUTDOWN_GRACE_SECS
                );
                tasks.abort_all();
                while tasks.join_next().await.is_some() {}
            }
        }

        println!("Server stopped");
    }
}
