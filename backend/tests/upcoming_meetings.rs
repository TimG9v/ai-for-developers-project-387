//! HTTP-шов: предстоящие встречи владельца — записи со временем слота
//! и типом встречи одним ответом; прошедшие слоты не отдаются.

mod common;

use std::sync::Arc;

use backend::api::api_types::{Booking, EventType, Slot};
use backend::domain::{BookingsRepository, EventTypesRepository, SlotsRepository};
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots};
use chrono::{Duration, Utc};

fn booking(id: &str, slot_id: &str, name: &str, email: &str) -> Booking {
    Booking {
        id: id.to_string(),
        slot_id: slot_id.to_string(),
        guest_name: name.to_string(),
        guest_email: email.to_string(),
        created_at: Utc::now(),
    }
}

/// Тип et1 (30 мин), предстоящий слот s1 (завтра) и прошедший s-past.
fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.add(EventType {
        id: "et1".to_string(),
        title: "Созвон".to_string(),
        description: None,
        duration_minutes: 30,
    });
    let slots = InMemorySlots::new();
    let start = Utc::now() + Duration::days(1);
    let s1 = Slot {
        id: "s1".to_string(),
        event_type_id: "et1".to_string(),
        start_date_time: start,
        end_date_time: start + Duration::minutes(30),
    };
    let s_past = Slot {
        id: "s-past".to_string(),
        event_type_id: "et1".to_string(),
        start_date_time: Utc::now() - Duration::days(30),
        end_date_time: Utc::now() - Duration::days(30) + Duration::minutes(30),
    };
    slots.add(s1.clone());
    slots.add(s_past.clone());
    let bookings = InMemoryBookings::new();
    bookings.try_add(booking("b1", "s1", "Гость", "g@example.com"), &s1);
    bookings.try_add(
        booking("b-past", "s-past", "Прошедший", "past@example.com"),
        &s_past,
    );
    backend::AppState {
        event_types: Arc::new(event_types),
        slots: Arc::new(slots),
        bookings: Arc::new(bookings),
    }
}

#[tokio::test]
async fn upcoming_meetings_join_booking_slot_and_event_type() {
    let app = backend::app_with_state(seeded_state());

    let raw = common::send(app, &common::get_request("/upcoming-meetings")).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let items: serde_json::Value =
        serde_json::from_str(common::response_body(&raw)).expect("JSON-тело");
    let items = items.as_array().expect("UpcomingMeeting[]");

    assert_eq!(items.len(), 1, "прошедшие встречи не отдаются: {items:?}");
    assert_eq!(items[0]["id"], "b1");
    assert_eq!(items[0]["guestName"], "Гость");
    assert_eq!(items[0]["guestEmail"], "g@example.com");
    assert_eq!(items[0]["eventTypeId"], "et1");
    assert_eq!(items[0]["eventTitle"], "Созвон");
    let start = items[0]["startDateTime"]
        .as_str()
        .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        .map(|parsed| parsed.with_timezone(&Utc))
        .expect("startDateTime в RFC3339");
    let expected_start = Utc::now() + Duration::days(1);
    assert!(
        (start - expected_start).num_milliseconds().abs() < 1_000,
        "время слота из ответа: {start}, ожидался слот на завтра: {expected_start}"
    );
}

#[tokio::test]
async fn upcoming_meetings_on_empty_storage_returns_empty_array() {
    let raw = common::send(backend::app(), &common::get_request("/upcoming-meetings")).await;

    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    assert_eq!(common::response_body(&raw).trim(), "[]");
}
