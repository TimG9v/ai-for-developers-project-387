//! HTTP-шов: запись гостя на слот. Атомарность слота (одна Запись на Слот),
//! конфликты 409, несуществующий слот 404, обязательность гостя 400,
//! единое окно 14 дней. Занятые слоты исчезают из календаря.

mod common;

use std::sync::Arc;

use backend::api::api_types::{Booking, EventType, Slot};
use backend::domain::{BookingsRepository, EventTypesRepository, SlotsRepository};
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots};
use chrono::{Duration, Utc};

fn event_type(id: &str, duration_minutes: i32) -> EventType {
    EventType {
        id: id.to_string(),
        title: "Созвон".to_string(),
        description: None,
        duration_minutes,
    }
}

fn slot(id: &str, event_type_id: &str, start: chrono::DateTime<Utc>, minutes: i64) -> Slot {
    Slot {
        id: id.to_string(),
        event_type_id: event_type_id.to_string(),
        start_date_time: start,
        end_date_time: start + Duration::minutes(minutes),
    }
}

/// Типы et1 (30 мин) и et2 (60 мин); свободные слоты s1 и s2 на завтра;
/// слот s-out на 15-й день (вне окна) — сеян напрямую мимо валидации.
fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.add(event_type("et1", 30));
    event_types.add(event_type("et2", 60));
    let slots = InMemorySlots::new();
    let start = Utc::now() + Duration::days(1);
    slots.add(slot("s1", "et1", start, 30));
    slots.add(slot("s2", "et2", start, 60));
    slots.add(slot("s-out", "et1", start + Duration::days(14), 30));
    backend::AppState {
        event_types: Arc::new(event_types),
        slots: Arc::new(slots),
        bookings: Arc::new(InMemoryBookings::new()),
    }
}

fn booking_body(id: &str, slot_id: &str, guest_name: &str, guest_email: &str) -> String {
    format!(
        r#"{{"id":"{id}","slotId":"{slot_id}","guestName":"{guest_name}","guestEmail":"{guest_email}","createdAt":"{}"}}"#,
        Utc::now().to_rfc3339()
    )
}

#[tokio::test]
async fn bookings_create_returns_created_booking_and_slot_leaves_calendar() {
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(
        app.clone(),
        &common::post_request(
            "/bookings",
            &booking_body("b1", "s1", "Гость", "g@example.com"),
        ),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let created: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(created["slotId"], "s1");
    assert_eq!(created["guestName"], "Гость");
    assert_eq!(created["guestEmail"], "g@example.com");

    // Занятый слот больше не отдаётся календарём записи.
    let raw = common::send(app, &common::get_request("/slots?eventTypeId=et1")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let ids: Vec<&str> = items
        .as_array()
        .expect("Slot[]")
        .iter()
        .map(|s| s["id"].as_str().expect("id"))
        .collect();
    assert!(
        !ids.contains(&"s1"),
        "занятый слот недоступен в календаре: {items}"
    );
}

#[tokio::test]
async fn bookings_create_second_booking_on_same_slot_returns_409() {
    let app = backend::app_with_state(seeded_state());

    // 409 независимо от типа встречи: проверяем на двух типах.
    for slot_id in ["s1", "s2"] {
        let first = common::send(
            app.clone(),
            &common::post_request(
                "/bookings",
                &booking_body(
                    &format!("b-first-{slot_id}"),
                    slot_id,
                    "Первый",
                    "first@example.com",
                ),
            ),
        )
        .await;
        assert!(first.contains("HTTP/1.1 200"), "{slot_id}: got: {first}");

        let second = common::send(
            app.clone(),
            &common::post_request(
                "/bookings",
                &booking_body(
                    &format!("b-second-{slot_id}"),
                    slot_id,
                    "Второй",
                    "second@example.com",
                ),
            ),
        )
        .await;
        assert!(
            second.contains("HTTP/1.1 409"),
            "повторная запись на {slot_id} должна быть 409, got: {second}"
        );
    }
}

#[tokio::test]
async fn bookings_create_unknown_slot_returns_404() {
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(
        app,
        &common::post_request(
            "/bookings",
            &booking_body("b1", "nope", "Гость", "g@example.com"),
        ),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 404"), "got: {raw}");
}

#[tokio::test]
async fn bookings_create_rejects_missing_guest_fields_with_400() {
    let invalid_bodies = [
        // пустое имя
        booking_body("b1", "s1", "", "g@example.com"),
        // имя из пробелов
        booking_body("b2", "s1", "   ", "g@example.com"),
        // пустой email
        booking_body("b3", "s1", "Гость", ""),
        // отсутствуют поля гостя
        r#"{"id":"b4","slotId":"s1","createdAt":"2026-10-01T09:00:00Z"}"#.to_string(),
        // вовсе не JSON
        "not json".to_string(),
    ];
    let app = backend::app_with_state(seeded_state());

    for body in invalid_bodies {
        let raw = common::send(app.clone(), &common::post_request("/bookings", &body)).await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "тело {body} должно отклоняться 400, got: {raw}"
        );
    }
}

#[tokio::test]
async fn bookings_create_rejects_slot_outside_window_with_400() {
    // История 16: окно едино для всех — прямой запрос на слот вне окна
    // отклоняется, даже если слот есть в хранилище.
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(
        app,
        &common::post_request(
            "/bookings",
            &booking_body("b1", "s-out", "Гость", "g@example.com"),
        ),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 400"), "got: {raw}");
}

#[test]
fn in_memory_bookings_try_add_is_insert_if_absent() {
    // Атомарность слота: проверка занятости и вставка — единая операция.
    let bookings = InMemoryBookings::new();
    let booking = Booking {
        id: "b1".to_string(),
        slot_id: "s1".to_string(),
        guest_name: "Гость".to_string(),
        guest_email: "g@example.com".to_string(),
        created_at: Utc::now(),
    };

    assert!(bookings.try_add(booking.clone()));
    assert!(!bookings.try_add(booking));
}

#[tokio::test]
async fn bookings_list_returns_bookings_of_all_types() {
    // Ракурс владельца: записи на слоты разных типов — в одном списке.
    let state = seeded_state();
    state.bookings.try_add(Booking {
        id: "b1".to_string(),
        slot_id: "s1".to_string(),
        guest_name: "Первый".to_string(),
        guest_email: "first@example.com".to_string(),
        created_at: Utc::now(),
    });
    state.bookings.try_add(Booking {
        id: "b2".to_string(),
        slot_id: "s2".to_string(),
        guest_name: "Второй".to_string(),
        guest_email: "second@example.com".to_string(),
        created_at: Utc::now(),
    });
    let app = backend::app_with_state(state);

    let raw = common::send(app, &common::get_request("/bookings")).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = items.as_array().expect("Booking[]");
    assert_eq!(items.len(), 2);
    let slot_ids: Vec<&str> = items
        .iter()
        .map(|booking| booking["slotId"].as_str().expect("slotId"))
        .collect();
    assert_eq!(slot_ids, vec!["s1", "s2"], "записи всех типов: {items:?}");
    assert_eq!(items[0]["guestName"], "Первый");
    assert_eq!(items[1]["guestEmail"], "second@example.com");
}

#[tokio::test]
async fn bookings_list_on_empty_storage_returns_empty_array() {
    let raw = common::send(backend::app(), &common::get_request("/bookings")).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    assert_eq!(common::response_body(&raw).trim(), "[]");
}
