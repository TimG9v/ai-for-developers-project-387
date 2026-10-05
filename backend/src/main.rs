#[tokio::main]
async fn main() {
    let backend_port = std::env::var("BACKEND_PORT").ok();
    let bind_addr = backend::bind_addr(backend_port.as_deref());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|err| panic!("bind {bind_addr}: {err}"));
    let app = backend::app();
    axum::serve(listener, app).await.expect("serve");
}
