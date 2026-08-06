use crate::{
    middleware::{CatchPanic, Cors, Middleware, RequestId, RequestLogger, SecurityHeaders},
    router::Router,
    server::handle_client::handle_client,
    state::State,
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Semaphore};

pub mod handle_client;

pub struct Server {
    pub listener: TcpListener,
    pub max_connections: u16,
    pub router: Router,
    pub middlewares: Vec<Arc<dyn Middleware>>,
    pub state: State,
}

impl Server {
    pub async fn bind(address: String) -> Self {
        Self {
            listener: Result::expect(
                TcpListener::bind(address).await,
                "Unable to initialize tcp listener",
            ),
            max_connections: 2,
            router: Router::new(),
            middlewares: vec![
                Arc::new(RequestId),
                Arc::new(RequestLogger),
                Arc::new(Cors::new()),
                Arc::new(SecurityHeaders),
                Arc::new(CatchPanic),
            ],
            state: State::new(),
        }
    }

    pub fn max_connects(self, connections: u16) -> Self {
        Self {
            max_connections: connections,
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

    pub fn state<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        let temp_state = self.state.insert(value);
        self.state = temp_state;
        self
    }

    pub async fn serve(self) {
        let admission = Arc::new(Semaphore::new(self.max_connections as usize));
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let state = Arc::new(self.state);

        loop {
            let stream_result = self.listener.accept().await;

            match stream_result {
                Ok((stream, _)) => {
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
                    tokio::spawn(async move {
                        let _permit = permit;
                        handle_client(stream, router, middlewares, state).await;
                    });
                }
                Err(_error) => {}
            }
        }
    }
}
