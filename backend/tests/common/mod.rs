//! Общий хелпер HTTP-шва: запрос к Router на реальном сокете (случайный порт).

#![allow(dead_code)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn send(router: axum::Router, request: &str) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind to random port");
    let addr = listener
        .local_addr()
        .expect("read actual port from listener");

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect to server");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("read response");
    String::from_utf8_lossy(&response).into_owned()
}

pub fn get_request(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
}

pub fn post_request(path: &str, body: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

pub fn response_body(raw: &str) -> &str {
    raw.split("\r\n\r\n").nth(1).expect("response body")
}
