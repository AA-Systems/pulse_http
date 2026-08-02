use std::sync::Arc;

use crate::server::handle_client::handle_client;
use tokio::{net::TcpListener, sync::Semaphore};
pub mod handle_client;

pub struct Server {
    pub listener: TcpListener,
    pub max_connections: u16,
}

impl Server {
    pub async fn bind(address: String) -> Self {
        Self {
            listener: Result::expect(
                TcpListener::bind(address).await,
                "Unable to initialize tcp listener",
            ),
            max_connections: 2,
        }
    }

    pub fn max_connects(self, connections: u16) -> Self {
        Self {
            listener: self.listener,
            max_connections: connections,
        }
    }

    pub async fn serve(self) {
        let adminssion = Arc::new(Semaphore::new(self.max_connections as usize));

        loop {
            let stream_result = self.listener.accept().await;

            match stream_result {
                Ok((stream, _)) => {
                    let permit = match adminssion.clone().try_acquire_owned() {
                        Ok(p) => p,
                        Err(_error) => {
                            drop(stream);
                            continue;
                        }
                    };
                    tokio::spawn(async move {
                        let _permit = permit;
                        handle_client(stream).await;
                    });
                }
                Err(_error) => {}
            }
        }
    }
}
