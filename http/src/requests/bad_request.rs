use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn bad_request(stream: &mut TcpStream) {
    let _ = stream
        .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
        .await;
}
