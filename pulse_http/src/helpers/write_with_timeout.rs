use std::time::Duration;
use tokio::{io::AsyncWriteExt, net::TcpStream, time::timeout};

pub async fn write_with_timeout(
    stream: &mut TcpStream,
    buf: &[u8],
    write_timeout_sec: u64,
) -> Result<(), ()> {
    match timeout(
        Duration::from_secs(write_timeout_sec),
        stream.write_all(buf),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        _ => Err(()),
    }
}
