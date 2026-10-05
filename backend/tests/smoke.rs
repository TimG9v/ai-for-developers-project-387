use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn server_boots_and_health_returns_200() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind to random port");
    let addr = listener
        .local_addr()
        .expect("read actual port from listener");
    let app = backend::app();

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect to server");
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .expect("write request");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("read response");
    let response = String::from_utf8_lossy(&response);

    assert!(
        response.contains("HTTP/1.1 200"),
        "response must contain `HTTP/1.1 200`, got: {response}"
    );
    assert!(
        response.contains(r#""status":"ok""#),
        "response must contain health body {{\"status\":\"ok\"}}, got: {response}"
    );
}
