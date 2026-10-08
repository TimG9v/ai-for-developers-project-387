//! HTTP-шов: публикация слотов владельцем и календарь гостя.
//! Окно 14 дней — серверное правило; фильтр по типу; привязка к существующему типу.

mod common;

use std::sync::Arc;

use backend::api::api_types::{EventType, Slot};
use backend::domain::{EventTypesRepository, SlotsRepository};
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots};
use chrono::{Duration, Utc};

fn event_type(id: &str, title: &str, duration_minutes: i32) -> EventType {
    EventType {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        duration_minutes,
    }
}

fn slot(
    id: &str,
    event_type_id: &str,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> Slot {
    Slot {
        id: id.to_string(),
        event_type_id: event_type_id.to_string(),
        start_date_time: start,
        end_date_time: end,
    }
}

/// Состояние с двумя типами встреч: et1 (30 мин) и et2 (60 мин).
fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.try_add(event_type("et1", "Созвон", 30));
    event_types.try_add(event_type("et2", "Консультация", 60));
    backend::AppState {
        event_types: Arc::new(event_types),
        slots: Arc::new(InMemorySlots::new()),
        bookings: Arc::new(InMemoryBookings::new()),
    }
}

fn slot_body(
    event_type_id: &str,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> String {
    format!(
        r#"{{"id":"s1","eventTypeId":"{event_type_id}","startDateTime":"{}","endDateTime":"{}"}}"#,
        start.to_rfc3339(),
        end.to_rfc3339()
    )
}

/// Привязка вниз к 30-минутной сетке — общий хелпер common::floor_grid.
#[tokio::test]
async fn slots_create_rejects_off_grid_start_with_400() {
    // Обязательное требование: шаг слотов 30 минут. 10:17 и :00:15 — вне сетки.
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());
    let base = common::floor_grid(Utc::now() + Duration::days(1));

    let cases: Vec<(String, &str)> = vec![
        (
            slot_body(
                "et1",
                base + Duration::minutes(17),
                base + Duration::minutes(47),
            ),
            "начало :17",
        ),
        (
            slot_body(
                "et1",
                base + Duration::seconds(15),
                base + Duration::minutes(30) + Duration::seconds(15),
            ),
            "начало :00:15",
        ),
        (
            slot_body(
                "et1",
                base + Duration::milliseconds(500),
                base + Duration::minutes(30) + Duration::milliseconds(500),
            ),
            "начало :00.500",
        ),
    ];

    for (body, label) in cases {
        let raw = common::send(app.clone(), &common::post_request("/slots", &body)).await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "слот вне сетки ({label}) должен отклоняться 400, got: {raw}"
        );
    }
}

#[tokio::test]
async fn slots_create_returns_created_slot_and_it_is_listed() {
    let state = seeded_state();
    let start = common::floor_grid(Utc::now() + Duration::days(1));
    let end = start + Duration::minutes(30);
    let app = backend::app_with_state(state.clone());

    let raw = common::send(
        app.clone(),
        &common::post_request("/slots", &slot_body("et1", start, end)),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let created: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(created["eventTypeId"], "et1");

    let raw = common::send(app, &common::get_request("/slots?eventTypeId=et1")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(items.as_array().expect("Slot[]").len(), 1);
    assert_eq!(items[0]["id"], "s1");
    let listed_start = items[0]["startDateTime"]
        .as_str()
        .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        .expect("startDateTime в RFC3339");
    assert_eq!(listed_start, start);
}

#[tokio::test]
async fn slots_create_unknown_event_type_returns_404() {
    let state = seeded_state();
    let start = Utc::now() + Duration::days(1);
    let end = start + Duration::minutes(30);
    let app = backend::app_with_state(state.clone());

    let raw = common::send(
        app.clone(),
        &common::post_request("/slots", &slot_body("nope", start, end)),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 404"), "got: {raw}");

    // Пустой eventTypeId — «тип не задан» — тоже несуществующий тип.
    let raw = common::send(
        app,
        &common::post_request("/slots", &slot_body("", start, end)),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 404"), "got: {raw}");
}

#[tokio::test]
async fn slots_create_rejects_out_of_window_and_bad_intervals_with_400() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());
    let now = Utc::now();

    let cases: Vec<(String, &str)> = vec![
        // 15-й день — за границей окна
        (
            slot_body(
                "et1",
                now + Duration::days(15),
                now + Duration::days(15) + Duration::minutes(30),
            ),
            "15-й день",
        ),
        // в прошлом
        (
            slot_body("et1", now - Duration::hours(1), now - Duration::minutes(30)),
            "в прошлом",
        ),
        // конец не позже начала
        (
            slot_body(
                "et1",
                now + Duration::days(1),
                now + Duration::days(1) - Duration::minutes(1),
            ),
            "end <= start",
        ),
    ];

    for (body, label) in cases {
        let raw = common::send(app.clone(), &common::post_request("/slots", &body)).await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "слот ({label}) должен отклоняться 400, got: {raw}"
        );
    }

    let raw = common::send(app, &common::get_request("/slots")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(
        items.as_array().expect("Slot[]").len(),
        0,
        "отклонённые слоты не сохраняются"
    );
}

#[tokio::test]
async fn slots_create_accepts_day_14_boundary() {
    let state = seeded_state();
    // Слот, начинающийся в 14-й день, — валиден (решение сессии);
    // floor_grid(now) + 14d <= now + 14d — граница окна соблюдена.
    let start = common::floor_grid(Utc::now()) + Duration::days(14);
    let end = start + Duration::minutes(30);
    let app = backend::app_with_state(state);

    let raw = common::send(
        app,
        &common::post_request("/slots", &slot_body("et1", start, end)),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
}

#[tokio::test]
async fn slots_create_rejects_duration_mismatch_with_type_400() {
    // Словарь: «длительность слота определяется его типом встречи» —
    // сервер обязан отклонять интервал, не равный длительности типа.
    let state = seeded_state();
    let start = Utc::now() + Duration::days(1);
    let app = backend::app_with_state(state.clone());

    let body = slot_body(
        "et1",
        start,
        start + Duration::minutes(45), // у et1 длительность 30 мин
    );
    let raw = common::send(app, &common::post_request("/slots", &body)).await;

    assert!(
        raw.contains("HTTP/1.1 400"),
        "интервал не по длительности типа должен отклоняться 400, got: {raw}"
    );
}

#[tokio::test]
async fn slots_create_rejects_content_duplicate_with_409() {
    // Правило дубля — по содержимому (тип встречи + время начала; конец
    // детерминирован длительностью типа), не по id: double-click формы шлёт
    // разные id, но повтор обязан отклоняться 409 и не попадать в список.
    let state = seeded_state();
    let start = common::floor_grid(Utc::now() + Duration::days(1));
    let end = start + Duration::minutes(30);
    let app = backend::app_with_state(state.clone());

    let first = slot_body("et1", start, end);
    let raw = common::send(app.clone(), &common::post_request("/slots", &first)).await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    // Другой id, то же содержимое — сетевой повтор/double-click.
    let duplicate = first.replace(r#""id":"s1""#, r#""id":"s2""#);
    let raw = common::send(app.clone(), &common::post_request("/slots", &duplicate)).await;
    assert!(
        raw.contains("HTTP/1.1 409"),
        "повтор слота с тем же типом и началом должен отклоняться 409, got: {raw}"
    );

    let raw = common::send(app, &common::get_request("/slots?eventTypeId=et1")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = items.as_array().expect("Slot[]");
    assert_eq!(items.len(), 1, "дубль не попадает в список: {items:?}");
    assert_eq!(items[0]["id"], "s1", "остаётся первая копия");
}

#[tokio::test]
async fn slots_create_different_content_is_not_a_duplicate() {
    // Разное содержимое — другой тип встречи или другое время начала:
    // это разные слоты, ограничение касается только полных совпадений.
    let state = seeded_state();
    let start = common::floor_grid(Utc::now() + Duration::days(1));
    let end = start + Duration::minutes(30);
    let app = backend::app_with_state(state.clone());

    let first = slot_body("et1", start, end);
    let raw = common::send(app.clone(), &common::post_request("/slots", &first)).await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    // Тот же тип, другое начало; тот же тип, но et2 длительностью 60 мин.
    let other_start = slot_body("et1", start + Duration::hours(1), end + Duration::hours(1));
    let raw = common::send(app.clone(), &common::post_request("/slots", &other_start)).await;
    assert!(
        raw.contains("HTTP/1.1 200"),
        "другое начало — не дубль, got: {raw}"
    );

    let other_type = slot_body(
        "et2",
        start + Duration::minutes(30),
        start + Duration::minutes(90),
    );
    let raw = common::send(app.clone(), &common::post_request("/slots", &other_type)).await;
    assert!(
        raw.contains("HTTP/1.1 200"),
        "другой тип встречи — не дубль, got: {raw}"
    );

    let raw = common::send(app, &common::get_request("/slots")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(items.as_array().expect("Slot[]").len(), 3);
}

#[tokio::test]
async fn slots_list_returns_only_slots_of_type_within_window() {
    let state = seeded_state();
    let slots_repo = InMemorySlots::new();
    let now = Utc::now();
    // В окне, нужный тип
    slots_repo.try_add(slot(
        "s1",
        "et1",
        now + Duration::days(1),
        now + Duration::days(1) + Duration::minutes(30),
    ));
    // В окне, другой тип
    slots_repo.try_add(slot(
        "s2",
        "et2",
        now + Duration::days(2),
        now + Duration::days(2) + Duration::minutes(60),
    ));
    // Нужный тип, но 15-й день — вне окна
    slots_repo.try_add(slot(
        "s3",
        "et1",
        now + Duration::days(15),
        now + Duration::days(15) + Duration::minutes(30),
    ));
    // Нужный тип, но в прошлом
    slots_repo.try_add(slot(
        "s4",
        "et1",
        now - Duration::hours(2),
        now - Duration::hours(2) + Duration::minutes(30),
    ));
    // Нужный тип, в окне на границе — 14-й день виден в списке
    slots_repo.try_add(slot(
        "s5",
        "et1",
        now + Duration::days(14),
        now + Duration::days(14) + Duration::minutes(30),
    ));
    let state = backend::AppState {
        event_types: state.event_types,
        slots: Arc::new(slots_repo),
        bookings: state.bookings,
    };
    let app = backend::app_with_state(state);

    let raw = common::send(app.clone(), &common::get_request("/slots?eventTypeId=et1")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let ids: Vec<&str> = items
        .as_array()
        .expect("Slot[]")
        .iter()
        .map(|s| s["id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        ids,
        vec!["s1", "s5"],
        "только слоты выбранного типа в окне, 14-й день виден: {items}"
    );

    let raw = common::send(app, &common::get_request("/slots?eventTypeId=ghost")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(items.as_array().expect("Slot[]").len(), 0);
}
