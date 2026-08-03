use crate::constants::READ_TIMEOUT_SECS;
use std::time::Duration;
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

pub async fn read_with_timeout(stream: &mut TcpStream, buf: &mut [u8]) -> Result<usize, ()> {
    match timeout(Duration::from_secs(READ_TIMEOUT_SECS), stream.read(buf)).await {
        Ok(Ok(n)) => Ok(n),
        _ => Err(()),
    }
}
