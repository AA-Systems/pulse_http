use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];
    let request = stream.read(&mut buffer).await;

    match request {
        Ok(_) => {
            println!("{:?}", String::from_utf8_lossy(&buffer));
        }
        Err(_error) => {}
    }

    let response = "HTTP/1.1 200 ok \r\nContent-Type: application/json; charset=utf-8 \r\nDate: Sun, 02 Aug 2026 07:30:55 GMT \r\nContent-Length: 2 \r\n\r\nok";

    let _ = stream.write(response.as_bytes()).await;
}
