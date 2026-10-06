use std::sync::Arc;

#[tokio::main]
async fn main() {
    let backend_port = std::env::var("BACKEND_PORT").ok();
    let bind_addr = backend::bind_addr(backend_port.as_deref());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|err| panic!("bind {bind_addr}: {err}"));
    // Состояние бинарника: in-memory + демо-данные (отключаются
    // BACKEND_SEED_DEMO=0). Тесты собирают состояние сами — без демо-данных.
    let state = backend::AppState {
        event_types: Arc::new(backend::infra::InMemoryEventTypes::new()),
        slots: Arc::new(backend::infra::InMemorySlots::new()),
        bookings: Arc::new(backend::infra::InMemoryBookings::new()),
    };
    if std::env::var("BACKEND_SEED_DEMO").as_deref() != Ok("0") {
        backend::seed::seed_demo(state.event_types.as_ref(), state.slots.as_ref());
    }
    let app = backend::app_with_state(state);
    axum::serve(listener, app).await.expect("serve");
}
