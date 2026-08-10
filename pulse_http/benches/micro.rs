use criterion::{Criterion, criterion_group, criterion_main};
use pulse_http::{
    Request, Response, Router, State, headers::Headers, parse_multipart, parse_urlencoded,
};
use std::{hint::black_box, net::IpAddr};

fn sample_headers() -> &'static [u8] {
    b"GET /hello/akshat?x=1 HTTP/1.1\r\n\
Host: localhost:3000\r\n\
User-Agent: pulse-bench\r\n\
Accept: */*\r\n\
Connection: keep-alive\r\n\
Content-Type: application/json\r\n\
\r\n"
}

fn sample_multipart() -> (&'static [u8], &'static str) {
    let body = b"\
--abc\r\n\
Content-Disposition: form-data; name=\"password\"\r\n\
\r\n\
secret\r\n\
--abc\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"note.txt\"\r\n\
Content-Type: text/plain\r\n\
\r\n\
hello world file bytes\r\n\
--abc--\r\n";
    (body, "multipart/form-data; boundary=abc")
}

fn bench_parse_headers(c: &mut Criterion) {
    let raw = sample_headers();
    c.bench_function("parse_headers", |b| {
        b.iter(|| Headers::parse(black_box(raw)).unwrap())
    });
}

fn bench_parse_urlencoded(c: &mut Criterion) {
    let body = "name=akshat&email=a%40b.com&msg=hello+world&count=42";
    c.bench_function("parse_urlencoded", |b| {
        b.iter(|| parse_urlencoded(black_box(body)).unwrap())
    });
}

fn bench_parse_multipart(c: &mut Criterion) {
    let (body, content_type) = sample_multipart();
    c.bench_function("parse_multipart", |b| {
        b.iter(|| parse_multipart(black_box(body), black_box(content_type)).unwrap())
    });
}

fn bench_router_match(c: &mut Criterion) {
    let router = Router::new()
        .get("/health", |_req: Request| async { Response::ok("ok") })
        .get("/hello/:name", |_req: Request| async { Response::ok("hi") })
        .post("/echo", |_req: Request| async { Response::ok("echo") });

    let raw = b"GET /hello/akshat HTTP/1.1\r\nHost: x\r\n\r\n";
    let headers = Headers::parse(raw).unwrap();
    let ip: IpAddr = "127.0.0.1".parse().unwrap();

    c.bench_function("router_match_param", |b| {
        b.iter(|| {
            let mut req =
                Request::from_headers(headers.clone(), Vec::new(), State::new(), ip).unwrap();
            router.match_route(black_box(&mut req)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_parse_headers,
    bench_parse_urlencoded,
    bench_parse_multipart,
    bench_router_match
);
criterion_main!(benches);
