//! Контрактные типы собираются и парсят форму из OpenAPI-спеки.
//! Генерат (`api_types`) руками не правится — только через контракт.

use serde::Deserialize;

#[test]
fn event_type_matches_contract_shape() {
    let raw = r#"{"id":"et1","title":"Звонок","durationMinutes":30}"#;
    let parsed = backend::api::api_types::EventType::deserialize(
        &mut serde_json::Deserializer::from_str(raw),
    )
    .expect("EventType из формы контракта");
    assert_eq!(parsed.id, "et1");
    assert_eq!(parsed.title, "Звонок");
    assert_eq!(parsed.duration_minutes, 30);
    assert!(parsed.description.is_none());
}

#[test]
fn slot_maps_date_time_to_chrono() {
    let raw = r#"{"id":"s1","eventTypeId":"et1","startDateTime":"2026-10-01T10:00:00Z","endDateTime":"2026-10-01T10:30:00Z"}"#;
    let parsed =
        backend::api::api_types::Slot::deserialize(&mut serde_json::Deserializer::from_str(raw))
            .expect("Slot из формы контракта");
    assert_eq!(parsed.event_type_id, "et1");
    assert_eq!(
        parsed.start_date_time.to_rfc3339(),
        "2026-10-01T10:00:00+00:00"
    );
    assert_eq!(
        parsed.end_date_time.to_rfc3339(),
        "2026-10-01T10:30:00+00:00"
    );
}

#[test]
fn booking_requires_guest_fields() {
    // guestName/guestEmail обязательны (решение карты #6): форма без них — ошибка.
    let raw = r#"{"id":"b1","slotId":"s1","createdAt":"2026-10-01T09:00:00Z"}"#;
    let result =
        backend::api::api_types::Booking::deserialize(&mut serde_json::Deserializer::from_str(raw));
    assert!(result.is_err(), "запись без гостя не должна парситься");
}
