//! HTTP-шов: GET/POST /event-types — список и создание типов встреч в форме контракта.
//! Хранение in-memory, состояние подставляется через app_with_state.

mod common;

use std::sync::Arc;

use backend::api::api_types::EventType;
use backend::domain::EventTypesRepository;
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots, InMemoryWorkingHours};

const EVENT_TYPES_REQUEST: &str = "/event-types";

#[tokio::test]
async fn event_types_list_returns_seeded_types_as_contract_json() {
    let repo = InMemoryEventTypes::new();
    repo.try_add(EventType {
        id: "et1".to_string(),
        title: "Знакомство".to_string(),
        description: Some("Первичный созвон".to_string()),
        duration_minutes: 30,
    });
    repo.try_add(EventType {
        id: "et2".to_string(),
        title: "Консультация".to_string(),
        description: None,
        duration_minutes: 60,
    });
    let app = backend::app_with_state(backend::AppState {
        event_types: Arc::new(repo),
        slots: Arc::new(InMemorySlots::new()),
        bookings: Arc::new(InMemoryBookings::new()),
        working_hours: Arc::new(InMemoryWorkingHours::new()),
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
async fn event_types_create_rejects_content_duplicate_with_409() {
    // Правило дубля — по содержимому (название + длительность), не по id:
    // double-click формы шлёт разные id, но одинаковый тип обязан
    // отклоняться 409 и не попадать в список.
    let app = backend::app();
    let first = r#"{"id":"et-a","title":"Созвон","durationMinutes":15}"#;
    let duplicate_other_id = r#"{"id":"et-b","title":"Созвон","durationMinutes":15}"#;

    let raw = common::send(
        app.clone(),
        &common::post_request(EVENT_TYPES_REQUEST, first),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    let raw = common::send(
        app.clone(),
        &common::post_request(EVENT_TYPES_REQUEST, duplicate_other_id),
    )
    .await;
    assert!(
        raw.contains("HTTP/1.1 409"),
        "повтор типа с тем же названием и длительностью должен отклоняться 409, got: {raw}"
    );

    let raw = common::send(app, &common::get_request(EVENT_TYPES_REQUEST)).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = items.as_array().expect("EventType[]");
    assert_eq!(items.len(), 1, "дубль не попадает в список: {items:?}");
    assert_eq!(items[0]["id"], "et-a", "остаётся первая копия");
}

#[tokio::test]
async fn event_types_create_duplicate_key_ignores_title_whitespace() {
    // Название сравнивается без краевых пробелов — форма шлёт уже
    // обрезанное значение, а прямой запрос с пробелами не обходит правило.
    let app = backend::app();
    let first = r#"{"id":"et-a","title":"Созвон","durationMinutes":15}"#;
    let padded = r#"{"id":"et-b","title":"  Созвон  ","durationMinutes":15}"#;

    let raw = common::send(
        app.clone(),
        &common::post_request(EVENT_TYPES_REQUEST, first),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    let raw = common::send(app, &common::post_request(EVENT_TYPES_REQUEST, padded)).await;
    assert!(
        raw.contains("HTTP/1.1 409"),
        "краевые пробелы не обходят правило дубля, got: {raw}"
    );
}

#[tokio::test]
async fn event_types_create_different_content_is_not_a_duplicate() {
    // Разное содержимое (название или длительность) — разные типы,
    // осознанное ограничение демки касается только полных совпадений.
    let app = backend::app();
    let first = r#"{"id":"et-a","title":"Созвон","durationMinutes":15}"#;
    let other_title = r#"{"id":"et-b","title":"Консультация","durationMinutes":15}"#;
    let other_duration = r#"{"id":"et-c","title":"Созвон","durationMinutes":30}"#;

    let raw = common::send(
        app.clone(),
        &common::post_request(EVENT_TYPES_REQUEST, first),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    for body in [other_title, other_duration] {
        let raw = common::send(
            app.clone(),
            &common::post_request(EVENT_TYPES_REQUEST, body),
        )
        .await;
        assert!(
            raw.contains("HTTP/1.1 200"),
            "тело {body} отличается содержимым и не дубль, got: {raw}"
        );
    }

    let raw = common::send(app, &common::get_request(EVENT_TYPES_REQUEST)).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(items.as_array().expect("EventType[]").len(), 3);
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
