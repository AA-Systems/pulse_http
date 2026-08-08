use std::time::Duration;
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

pub async fn read_with_timeout(
    stream: &mut TcpStream,
    buf: &mut [u8],
    read_timeout_sec: u64,
) -> Result<usize, ()> {
    match timeout(Duration::from_secs(read_timeout_sec), stream.read(buf)).await {
        Ok(Ok(n)) => Ok(n),
        _ => Err(()),
    }
}
