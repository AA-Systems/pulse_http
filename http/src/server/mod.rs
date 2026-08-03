use crate::{router::Router, server::handle_client::handle_client};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Semaphore};

pub mod handle_client;

pub struct Server {
    pub listener: TcpListener,
    pub max_connections: u16,
    pub router: Router,
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

    pub async fn serve(self) {
        let admission = Arc::new(Semaphore::new(self.max_connections as usize));
        let router = Arc::new(self.router);

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
                    tokio::spawn(async move {
                        let _permit = permit;
                        handle_client(stream, router).await;
                    });
                }
                Err(_error) => {}
            }
        }
    }
}
