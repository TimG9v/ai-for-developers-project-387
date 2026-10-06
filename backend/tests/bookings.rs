//! HTTP-шов: запись гостя на слот. Атомарность слота (одна Запись на Слот),
//! конфликты 409, несуществующий слот 404, обязательность гостя 400,
//! единое окно 14 дней. Занятые слоты исчезают из календаря.

mod common;

use std::sync::Arc;

use backend::api::api_types::{Booking, EventType, Slot};
use backend::domain::{BookingsRepository, EventTypesRepository, RescheduleError, SlotsRepository};
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

/// Типы et1 (30 мин) и et2 (60 мин); свободные слоты s1 и s3 (оба et1) и
/// s2 (et2) на завтра; слот s-out на 15-й день (вне окна) — сеян напрямую
/// мимо валидации. Два слота одного типа — для сценариев переноса.
fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.add(event_type("et1", 30));
    event_types.add(event_type("et2", 60));
    let slots = InMemorySlots::new();
    let start = Utc::now() + Duration::days(1);
    slots.add(slot("s1", "et1", start, 30));
    slots.add(slot("s2", "et2", start, 60));
    slots.add(slot("s3", "et1", start + Duration::hours(2), 30));
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

async fn seed_booking(app: &axum::Router, id: &str, slot_id: &str) {
    let raw = common::send(
        app.clone(),
        &common::post_request(
            "/bookings",
            &booking_body(id, slot_id, "Гость", "g@example.com"),
        ),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "seed {id}: got: {raw}");
}

fn calendar_slot_ids(raw: &str) -> Vec<String> {
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(raw)).expect("JSON-тело");
    items
        .as_array()
        .expect("Slot[]")
        .iter()
        .map(|slot| slot["id"].as_str().expect("id").to_string())
        .collect()
}

#[tokio::test]
async fn bookings_cancel_returns_204_and_frees_the_slot() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;

    let raw = common::send(app.clone(), &common::delete_request("/bookings/b1")).await;
    assert!(raw.contains("HTTP/1.1 204"), "got: {raw}");
    assert_eq!(common::response_body(&raw).trim(), "");

    // Отменённый слот снова виден в календаре записи (история 7).
    let raw = common::send(app.clone(), &common::get_request("/slots?eventTypeId=et1")).await;
    assert!(
        calendar_slot_ids(&raw).contains(&"s1".to_string()),
        "слот должен освободиться: {raw}"
    );

    // Повторная отмена той же записи — записи больше нет.
    let raw = common::send(app, &common::delete_request("/bookings/b1")).await;
    assert!(raw.contains("HTTP/1.1 404"), "got: {raw}");
}

#[tokio::test]
async fn bookings_cancel_unknown_booking_returns_404() {
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(app, &common::delete_request("/bookings/nope")).await;

    assert!(raw.contains("HTTP/1.1 404"), "got: {raw}");
}

#[tokio::test]
async fn bookings_reschedule_moves_booking_to_new_slot_of_same_type() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;

    let raw = common::send(
        app.clone(),
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"s3"}"#),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let updated: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(updated["slotId"], "s3");
    assert_eq!(updated["guestName"], "Гость");

    // Старый слот освободился, новый занят.
    let raw = common::send(app.clone(), &common::get_request("/slots?eventTypeId=et1")).await;
    let ids = calendar_slot_ids(&raw);
    assert!(
        ids.contains(&"s1".to_string()),
        "старый слот свободен: {ids:?}"
    );
    assert!(
        !ids.contains(&"s3".to_string()),
        "новый слот занят: {ids:?}"
    );
}

#[tokio::test]
async fn bookings_reschedule_to_taken_slot_returns_409() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;
    seed_booking(&app, "b2", "s3").await;

    let raw = common::send(
        app,
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"s3"}"#),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 409"), "got: {raw}");
}

#[tokio::test]
async fn bookings_reschedule_to_other_event_type_returns_400() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;

    // s2 — слот типа et2, запись оформлена на et1.
    let raw = common::send(
        app,
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"s2"}"#),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 400"), "got: {raw}");
}

#[tokio::test]
async fn bookings_reschedule_to_slot_outside_window_returns_400() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;

    // История 16: окно едино для всех — перенос на слот вне окна отклоняется.
    let raw = common::send(
        app,
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"s-out"}"#),
    )
    .await;

    assert!(raw.contains("HTTP/1.1 400"), "got: {raw}");
}

#[tokio::test]
async fn bookings_reschedule_rejects_unknown_booking_and_unknown_slot_with_404() {
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(
        app.clone(),
        &common::post_request("/bookings/nope/reschedule", r#"{"newSlotId":"s3"}"#),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 404"), "нет записи: got: {raw}");

    seed_booking(&app, "b1", "s1").await;
    let raw = common::send(
        app,
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"nope"}"#),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 404"), "нет слота: got: {raw}");
}

#[tokio::test]
async fn bookings_reschedule_rejects_invalid_body_with_400() {
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await;

    let invalid_bodies = [
        // пустое тело без newSlotId
        "{}".to_string(),
        // вовсе не JSON
        "not json".to_string(),
    ];
    for body in invalid_bodies {
        let raw = common::send(
            app.clone(),
            &common::post_request("/bookings/b1/reschedule", &body),
        )
        .await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "тело {body} должно отклоняться 400, got: {raw}"
        );
    }
}

#[test]
fn in_memory_bookings_remove_removes_exactly_once() {
    let bookings = InMemoryBookings::new();
    bookings.try_add(Booking {
        id: "b1".to_string(),
        slot_id: "s1".to_string(),
        guest_name: "Гость".to_string(),
        guest_email: "g@example.com".to_string(),
        created_at: Utc::now(),
    });

    assert!(bookings.remove("b1"));
    assert!(!bookings.remove("b1"));
    assert!(!bookings.contains_slot("s1"), "слот освободился");
}

#[test]
fn in_memory_bookings_reschedule_rejects_taken_target_without_changes() {
    // Атомарность переноса: занятый целевой слот — отказ, запись на месте.
    let bookings = InMemoryBookings::new();
    for (id, slot_id) in [("b1", "s1"), ("b2", "s2")] {
        bookings.try_add(Booking {
            id: id.to_string(),
            slot_id: slot_id.to_string(),
            guest_name: "Гость".to_string(),
            guest_email: "g@example.com".to_string(),
            created_at: Utc::now(),
        });
    }

    assert!(matches!(
        bookings.reschedule("b1", "s2"),
        Err(RescheduleError::NewSlotTaken)
    ));
    assert!(bookings.contains_slot("s1"), "запись не сдвинулась");
    assert!(matches!(
        bookings.reschedule("nope", "s3"),
        Err(RescheduleError::BookingNotFound)
    ));

    let moved = bookings
        .reschedule("b1", "s3")
        .expect("перенос на свободный слот");
    assert_eq!(moved.slot_id, "s3");
    assert!(!bookings.contains_slot("s1"), "старый слот освободился");
}
