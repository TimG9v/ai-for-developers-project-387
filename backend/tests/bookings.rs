//! HTTP-шов: запись гостя на слот. Атомарность интервала (записи не
//! пересекаются по времени, ADR 0004), конфликты 409, несуществующий слот
//! 404, обязательность гостя 400, единое окно 14 дней. Слоты, занятые по
//! времени, исчезают из календаря.

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

/// Типы et1 (30 мин) и et2 (60 мин); свободные слоты s1 и s3 (оба et1),
/// s2 (et2, то же начало, что s1 — пересечение) и s4 (et2, стык с s1)
/// на завтра; слот s-out на 15-й день (вне окна) — сеян напрямую мимо
/// валидации. Два слота одного типа — для сценариев переноса.
fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.add(event_type("et1", 30));
    event_types.add(event_type("et2", 60));
    let slots = InMemorySlots::new();
    // Начало на 30-минутной сетке: seeded-слоты участвуют и в переносах,
    // где сетку проверяет validate_reschedule.
    let start = common::floor_grid(Utc::now() + Duration::days(1));
    slots.add(slot("s1", "et1", start, 30));
    slots.add(slot("s2", "et2", start, 60));
    slots.add(slot("s4", "et2", start + Duration::minutes(30), 60));
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

    seed_booking(&app, "b-first-s1", "s1").await;

    let second = common::send(
        app,
        &common::post_request(
            "/bookings",
            &booking_body("b-second-s1", "s1", "Второй", "second@example.com"),
        ),
    )
    .await;
    assert!(
        second.contains("HTTP/1.1 409"),
        "повторная запись на s1 должна быть 409, got: {second}"
    );
    // Пересечение слотов разных типов — отдельный тест (ADR 0004).
}

#[tokio::test]
async fn slots_list_hides_slots_overlapping_booked_interval() {
    // ADR 0004 в read-пути: календарь не предлагает слоты, чей интервал
    // пересекается с занятым, — даже слоты других типов; стык остаётся видим.
    let app = backend::app_with_state(seeded_state());
    seed_booking(&app, "b1", "s1").await; // et1: start..start+30

    let raw = common::send(app, &common::get_request("/slots?eventTypeId=et2")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let ids: Vec<&str> = items
        .as_array()
        .expect("Slot[]")
        .iter()
        .map(|slot| slot["id"].as_str().expect("id"))
        .collect();
    assert!(
        !ids.contains(&"s2"),
        "пересекающийся с бронью слот скрыт из календаря: {items}"
    );
    assert!(
        ids.contains(&"s4"),
        "слот, стыкующийся с бронью, доступен: {items}"
    );
}

#[tokio::test]
async fn bookings_create_on_overlapping_slot_of_other_event_type_returns_409() {
    // ADR 0004: занятость времени не зависит от типа встречи — слот другого
    // типа с пересекающимся интервалом отклоняется 409.
    let app = backend::app_with_state(seeded_state());

    seed_booking(&app, "b1", "s1").await; // et1: start..start+30

    // s2 (et2) стартует одновременно с s1 и длится 60 мин — пересечение.
    let raw = common::send(
        app,
        &common::post_request(
            "/bookings",
            &booking_body("b2", "s2", "Второй", "second@example.com"),
        ),
    )
    .await;
    assert!(
        raw.contains("HTTP/1.1 409"),
        "пересечение интервалов: {raw}"
    );
}

#[tokio::test]
async fn bookings_create_on_touching_slot_returns_200() {
    // Стык слотов (начало следующего == конец занятого) — не конфликт.
    let app = backend::app_with_state(seeded_state());

    seed_booking(&app, "b1", "s1").await; // start..start+30
    seed_booking(&app, "b2", "s4").await; // s4: start+30..start+90 (et2)

    let raw = common::send(app, &common::get_request("/bookings")).await;
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    assert_eq!(items.as_array().expect("Booking[]").len(), 2);
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
    // Атомарность интервала: вставка не создаёт пересекающихся записей.
    let bookings = InMemoryBookings::new();
    let start = Utc::now() + Duration::days(1);
    let booking = |id: &str, slot_id: &str| Booking {
        id: id.to_string(),
        slot_id: slot_id.to_string(),
        guest_name: "Гость".to_string(),
        guest_email: "g@example.com".to_string(),
        created_at: Utc::now(),
    };

    let s1 = slot("s1", "et1", start, 30);
    assert!(bookings.try_add(booking("b1", "s1"), &s1));
    // тот же слот — тот же интервал
    assert!(!bookings.try_add(booking("b2", "s1"), &s1));
    // другой тип, начало внутри занятого интервала — пересечение
    let s2 = slot("s2", "et2", start + Duration::minutes(15), 30);
    assert!(!bookings.try_add(booking("b3", "s2"), &s2));
    // стык с занятым слотом — не пересечение
    let s3 = slot("s3", "et1", start + Duration::minutes(30), 30);
    assert!(bookings.try_add(booking("b4", "s3"), &s3));
}

#[tokio::test]
async fn bookings_list_returns_bookings_of_all_types() {
    // Ракурс владельца: записи на слоты разных типов — в одном списке.
    let state = seeded_state();
    let start = Utc::now() + Duration::days(1);
    assert!(state.bookings.try_add(
        Booking {
            id: "b1".to_string(),
            slot_id: "s1".to_string(),
            guest_name: "Первый".to_string(),
            guest_email: "first@example.com".to_string(),
            created_at: Utc::now(),
        },
        &slot("s1", "et1", start, 30),
    ));
    assert!(state.bookings.try_add(
        Booking {
            id: "b2".to_string(),
            slot_id: "s4".to_string(),
            guest_name: "Второй".to_string(),
            guest_email: "second@example.com".to_string(),
            created_at: Utc::now(),
        },
        &slot("s4", "et2", start + Duration::minutes(30), 60),
    ));
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
    assert_eq!(slot_ids, vec!["s1", "s4"], "записи всех типов: {items:?}");
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
async fn bookings_reschedule_to_off_grid_slot_returns_400() {
    let state = seeded_state();
    // et1, начало через день + 17 минут от сетки — вне 30-минутной сетки;
    // сеян напрямую, поэтому сетку ловит только validate_reschedule.
    let base = common::floor_grid(Utc::now() + Duration::days(1));
    state
        .slots
        .add(slot("s-off-grid", "et1", base + Duration::minutes(17), 30));
    let app = backend::app_with_state(state);
    seed_booking(&app, "b1", "s1").await;

    let raw = common::send(
        app,
        &common::post_request("/bookings/b1/reschedule", r#"{"newSlotId":"s-off-grid"}"#),
    )
    .await;
    assert!(
        raw.contains("HTTP/1.1 400"),
        "вне сетки — 400, не 404: {raw}"
    );
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
    let start = Utc::now() + Duration::days(1);
    bookings.try_add(
        Booking {
            id: "b1".to_string(),
            slot_id: "s1".to_string(),
            guest_name: "Гость".to_string(),
            guest_email: "g@example.com".to_string(),
            created_at: Utc::now(),
        },
        &slot("s1", "et1", start, 30),
    );

    assert!(bookings.remove("b1"));
    assert!(!bookings.remove("b1"));
    assert!(!bookings.contains_slot("s1"), "слот освободился");
}

#[test]
fn in_memory_bookings_reschedule_rejects_taken_target_without_changes() {
    // Атомарность переноса: занятый целевой интервал — отказ, запись на месте.
    let bookings = InMemoryBookings::new();
    let start = Utc::now() + Duration::days(1);
    // Соседние непересекающиеся слоты: s1 и s2 заняты, s3 — свободный.
    let s1 = slot("s1", "et1", start, 30);
    let s2 = slot("s2", "et1", start + Duration::minutes(30), 30);
    let s3 = slot("s3", "et1", start + Duration::minutes(60), 30);
    for (id, target) in [("b1", &s1), ("b2", &s2)] {
        assert!(bookings.try_add(
            Booking {
                id: id.to_string(),
                slot_id: target.id.clone(),
                guest_name: "Гость".to_string(),
                guest_email: "g@example.com".to_string(),
                created_at: Utc::now(),
            },
            target,
        ));
    }

    assert!(matches!(
        bookings.reschedule("b1", &s2),
        Err(RescheduleError::NewSlotTaken)
    ));
    assert!(bookings.contains_slot("s1"), "запись не сдвинулась");
    assert!(matches!(
        bookings.reschedule("nope", &s3),
        Err(RescheduleError::BookingNotFound)
    ));

    let moved = bookings
        .reschedule("b1", &s3)
        .expect("перенос на свободный слот");
    assert_eq!(moved.slot_id, "s3");
    assert!(!bookings.contains_slot("s1"), "старый слот освободился");
}
