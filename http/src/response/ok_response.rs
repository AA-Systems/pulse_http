use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn ok_response(stream: &mut TcpStream) {
    let _ = stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nok")
        .await;
}
