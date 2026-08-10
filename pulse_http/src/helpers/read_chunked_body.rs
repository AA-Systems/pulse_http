use std::str::from_utf8;

use crate::helpers::{find_subslice::find_subslice, read_with_timeout::read_with_timeout};
use tokio::net::TcpStream;

pub async fn read_chunked_body(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    body_start: usize,
    max_body_size: usize,
    read_timeout_sec: u64,
) -> Result<(Vec<u8>, Vec<u8>), ()> {
    let mut cursor = body_start;
    let mut body = Vec::new();
    let mut read_buf = [0u8; 1024];

    loop {
        let size_line_end =
            ensure_crlf(stream, buffer, cursor, read_timeout_sec, &mut read_buf).await?;
        let size_line = from_utf8(&buffer[cursor..size_line_end]).map_err(|_| ())?;
        let size = parse_chunk_size(size_line)?;
        cursor = size_line_end + 2;

        if size == 0 {
            cursor = skip_trailers(stream, buffer, cursor, read_timeout_sec, &mut read_buf).await?;
            return Ok((body, buffer[cursor..].to_vec()));
        }

        let need = size.checked_add(2).ok_or(())?;
        ensure_len(
            stream,
            buffer,
            cursor + need,
            read_timeout_sec,
            &mut read_buf,
        )
        .await?;

        if body.len().saturating_add(size) > max_body_size {
            return Err(());
        }

        body.extend_from_slice(&buffer[cursor..cursor + size]);
        if &buffer[cursor + size..cursor + size + 2] != b"\r\n" {
            return Err(());
        }
        cursor += need;
    }
}

pub fn is_chunked_transfer_encoding(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("chunked")
}

fn parse_chunk_size(line: &str) -> Result<usize, ()> {
    let hex = line.split(';').next().unwrap_or(line).trim();
    if hex.is_empty() {
        return Err(());
    }
    usize::from_str_radix(hex, 16).map_err(|_| ())
}

async fn ensure_crlf(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    cursor: usize,
    read_timeout_sec: u64,
    read_buf: &mut [u8],
) -> Result<usize, ()> {
    loop {
        if let Some(rel) = find_subslice(&buffer[cursor..], b"\r\n") {
            return Ok(cursor + rel);
        }
        read_more(stream, buffer, read_timeout_sec, read_buf).await?;
    }
}

async fn ensure_len(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    needed: usize,
    read_timeout_sec: u64,
    read_buf: &mut [u8],
) -> Result<(), ()> {
    while buffer.len() < needed {
        read_more(stream, buffer, read_timeout_sec, read_buf).await?;
    }
    Ok(())
}

async fn skip_trailers(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    cursor: usize,
    read_timeout_sec: u64,
    read_buf: &mut [u8],
) -> Result<usize, ()> {
    loop {
        if buffer.len() >= cursor + 2 && &buffer[cursor..cursor + 2] == b"\r\n" {
            return Ok(cursor + 2);
        }
        if let Some(rel) = find_subslice(&buffer[cursor..], b"\r\n\r\n") {
            return Ok(cursor + rel + 4);
        }
        read_more(stream, buffer, read_timeout_sec, read_buf).await?;
    }
}

async fn read_more(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    read_timeout_sec: u64,
    read_buf: &mut [u8],
) -> Result<(), ()> {
    match read_with_timeout(stream, read_buf, read_timeout_sec).await {
        Ok(0) => Err(()),
        Ok(n) => {
            buffer.extend_from_slice(&read_buf[..n]);
            Ok(())
        }
        Err(()) => Err(()),
    }
}
