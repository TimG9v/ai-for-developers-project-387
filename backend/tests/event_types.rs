//! HTTP-шов: GET/POST /event-types — список и создание типов встреч в форме контракта.
//! Хранение in-memory, состояние подставляется через app_with_state.

mod common;

use std::sync::Arc;

use backend::api::api_types::EventType;
use backend::domain::EventTypesRepository;
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots};

const EVENT_TYPES_REQUEST: &str = "/event-types";

#[tokio::test]
async fn event_types_list_returns_seeded_types_as_contract_json() {
    let repo = InMemoryEventTypes::new();
    repo.add(EventType {
        id: "et1".to_string(),
        title: "Знакомство".to_string(),
        description: Some("Первичный созвон".to_string()),
        duration_minutes: 30,
    });
    repo.add(EventType {
        id: "et2".to_string(),
        title: "Консультация".to_string(),
        description: None,
        duration_minutes: 60,
    });
    let app = backend::app_with_state(backend::AppState {
        event_types: Arc::new(repo),
        slots: Arc::new(InMemorySlots::new()),
        bookings: Arc::new(InMemoryBookings::new()),
    });

    let raw = common::send(app, &common::get_request(EVENT_TYPES_REQUEST)).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let json: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = json.as_array().expect("EventType[]");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], "et1");
    assert_eq!(items[0]["title"], "Знакомство");
    assert_eq!(items[0]["description"], "Первичный созвон");
    assert_eq!(items[0]["durationMinutes"], 30);
    assert_eq!(items[1]["title"], "Консультация");
    assert_eq!(items[1]["durationMinutes"], 60);
    assert!(
        items[1].get("description").is_none(),
        "description без значения не сериализуется: {}",
        items[1]
    );
}

#[tokio::test]
async fn event_types_list_on_empty_storage_returns_empty_array() {
    let raw = common::send(backend::app(), &common::get_request(EVENT_TYPES_REQUEST)).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    assert_eq!(
        common::response_body(&raw).trim(),
        "[]",
        "пустое хранилище — валидный список"
    );
}

#[tokio::test]
async fn event_types_create_returns_created_type_and_it_is_listed() {
    let app = backend::app();
    let body = r#"{"id":"et2","title":"Созвон","durationMinutes":15}"#;

    let raw = common::send(
        app.clone(),
        &common::post_request(EVENT_TYPES_REQUEST, body),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let created: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(created["id"], "et2");
    assert_eq!(created["title"], "Созвон");
    assert_eq!(created["durationMinutes"], 15);

    let raw = common::send(app, &common::get_request(EVENT_TYPES_REQUEST)).await;
    let json: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = json.as_array().expect("EventType[]");
    assert_eq!(items.len(), 1, "созданный тип появляется в списке: {json}");
    assert_eq!(items[0]["title"], "Созвон");
}

#[tokio::test]
async fn event_types_create_rejects_invalid_input_with_400() {
    let invalid_bodies = [
        // пустое название
        r#"{"id":"e1","title":"","durationMinutes":30}"#,
        // название из пробелов
        r#"{"id":"e2","title":"   ","durationMinutes":30}"#,
        // нулевая длительность
        r#"{"id":"e3","title":"X","durationMinutes":0}"#,
        // отрицательная длительность
        r#"{"id":"e4","title":"X","durationMinutes":-5}"#,
        // отсутствуют обязательные поля
        r#"{}"#,
        r#"{"id":"e5","title":"X"}"#,
        // вовсе не JSON
        "not json",
    ];
    let app = backend::app();

    for body in invalid_bodies {
        let raw = common::send(
            app.clone(),
            &common::post_request(EVENT_TYPES_REQUEST, body),
        )
        .await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "тело {body} должно отклоняться 400, got: {raw}"
        );
    }

    let raw = common::send(app, &common::get_request(EVENT_TYPES_REQUEST)).await;
    assert_eq!(
        common::response_body(&raw).trim(),
        "[]",
        "отклонённые типы не должны попадать в список"
    );
}
