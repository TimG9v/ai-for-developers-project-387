//! HTTP-шов: рабочие часы владельца — сохранение расписания и материализация
//! слотов (issue #9). Идемпотентность, сужение окна, защита занятых слотов,
//! ручные слоты вне окон, валидация до мутации.

mod common;

use std::sync::Arc;

use backend::api::api_types::{EventType, Slot};
use backend::domain::{BOOKING_WINDOW_DAYS, EventTypesRepository};
use backend::infra::{InMemoryBookings, InMemoryEventTypes, InMemorySlots};
use chrono::{Datelike, Duration, TimeZone, Utc};

fn event_type(id: &str, duration_minutes: i32) -> EventType {
    EventType {
        id: id.to_string(),
        title: "Созвон".to_string(),
        description: None,
        duration_minutes,
    }
}

fn seeded_state() -> backend::AppState {
    let event_types = InMemoryEventTypes::new();
    event_types.try_add(event_type("et1", 30));
    event_types.try_add(event_type("et2", 60));
    backend::AppState {
        event_types: Arc::new(event_types),
        slots: Arc::new(InMemorySlots::new()),
        bookings: Arc::new(InMemoryBookings::new()),
        working_hours: Arc::new(backend::infra::InMemoryWorkingHours::new()),
    }
}

fn schedule_body(time_zone: &str, weekdays: &[i32], start: &str, end: &str) -> String {
    let weekdays = weekdays
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    if weekdays.is_empty() {
        return format!(r#"{{"timeZone":"{time_zone}","rules":[]}}"#);
    }
    format!(
        r#"{{"timeZone":"{time_zone}","rules":[{{"weekdays":[{weekdays}],"startTime":"{start}","endTime":"{end}"}}]}}"#
    )
}

async fn put_schedule(app: axum::Router, body: &str) -> String {
    common::send(app, &common::put_request("/working-hours", body)).await
}

/// Сколько дней горизонта (0..=14) попадают в окно с данным расписанием
/// в зоне UTC: день недели входит в `weekdays`, а начало окна лежит в
/// продовом окне записи `now <= start <= now + BOOKING_WINDOW_DAYS` —
/// то же условие, что в domain::is_within_booking_window, поэтому тест
/// не зависит от времени суток.
fn expected_window_days(weekdays: &[i32], wall_hour: u32) -> usize {
    let now = Utc::now();
    (0..=14)
        .filter(|offset| {
            let day = now.date_naive() + Duration::days(*offset);
            weekdays.contains(&(day.weekday().number_from_monday() as i32))
                && day
                    .and_hms_opt(wall_hour, 0, 0)
                    .and_then(|naive| Utc.from_local_datetime(&naive).single())
                    .is_some_and(|start| {
                        start >= now && start <= now + Duration::days(BOOKING_WINDOW_DAYS)
                    })
        })
        .count()
}

fn sorted_starts(slots: &[Slot], event_type_id: &str) -> Vec<chrono::DateTime<Utc>> {
    let mut starts: Vec<_> = slots
        .iter()
        .filter(|slot| slot.event_type_id == event_type_id)
        .map(|slot| slot.start_date_time)
        .collect();
    starts.sort();
    starts
}

#[tokio::test]
async fn working_hours_put_materializes_slots_and_get_roundtrips() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());
    let weekdays = [1, 2, 3, 4, 5];

    // GET до сохранения — пустое расписание по умолчанию.
    let raw = common::send(app.clone(), &common::get_request("/working-hours")).await;
    let empty: serde_json::Value = serde_json::from_str(common::response_body(&raw)).expect("JSON");
    assert_eq!(empty["rules"].as_array().map(Vec::len), Some(0));

    let raw = put_schedule(
        app.clone(),
        &schedule_body("UTC", &weekdays, "10:00", "17:00"),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    // GET возвращает сохранённое расписание.
    let raw = common::send(app.clone(), &common::get_request("/working-hours")).await;
    let saved: serde_json::Value = serde_json::from_str(common::response_body(&raw)).expect("JSON");
    assert_eq!(saved["timeZone"], "UTC");
    assert_eq!(saved["rules"][0]["startTime"], "10:00");
    assert_eq!(saved["rules"][0]["endTime"], "17:00");

    // Материализация: каждый рабочий день горизонта — 14 слотов 30-минутного
    // типа (10:00…16:30) и 7 слотов 60-минутного (10:00…16:00).
    let days = expected_window_days(&weekdays, 10);
    let slots = state.slots.list();
    assert_eq!(
        sorted_starts(&slots, "et1").len(),
        days * 14,
        "30-минутный тип: 14 слотов в день × {days} дней"
    );
    assert_eq!(
        sorted_starts(&slots, "et2").len(),
        days * 7,
        "60-минутный тип: 7 слотов в день × {days} дней"
    );

    // Структура дня: слоты встык от 10:00 до 17:00, начало на сетке.
    let tomorrow = (Utc::now() + Duration::days(1)).date_naive();
    let tomorrow_ten = tomorrow.and_hms_opt(10, 0, 0).unwrap();
    let starts = sorted_starts(&slots, "et1");
    let ten = Utc.from_local_datetime(&tomorrow_ten).single().unwrap();
    assert!(
        starts.contains(&ten),
        "завтра 10:00 UTC должно существовать"
    );
    let day_slots: Vec<_> = starts
        .iter()
        .filter(|start| **start >= ten && **start < ten + Duration::hours(7))
        .collect();
    assert_eq!(day_slots.len(), 14);
    for pair in day_slots.windows(2) {
        assert_eq!(*pair[1] - *pair[0], Duration::minutes(30), "встык");
    }
}

#[tokio::test]
async fn working_hours_put_is_idempotent() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());
    let body = schedule_body("UTC", &[1, 2, 3, 4, 5], "10:00", "12:00");

    put_schedule(app.clone(), &body).await;
    let mut first: Vec<_> = state.slots.list().iter().map(|s| s.id.clone()).collect();
    first.sort();

    put_schedule(app.clone(), &body).await;
    let mut second: Vec<_> = state.slots.list().iter().map(|s| s.id.clone()).collect();
    second.sort();

    assert_eq!(first, second, "повторный PUT не создаёт дубликатов");
}

#[tokio::test]
async fn narrowing_removes_unbooked_slots_and_keeps_booked_and_manual() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());

    // Полное окно: все дни 10:00–11:00 UTC → 30-минутный тип имеет 10:00 и 10:30.
    put_schedule(
        app.clone(),
        &schedule_body("UTC", &[1, 2, 3, 4, 5, 6, 7], "10:00", "11:00"),
    )
    .await;

    // Ручной слот вне окон (завтра 20:00 UTC) — разовый сценарий.
    let manual_start = (Utc::now() + Duration::days(1))
        .date_naive()
        .and_hms_opt(20, 0, 0)
        .unwrap();
    let manual_start = Utc.from_local_datetime(&manual_start).single().unwrap();
    let manual_body = format!(
        r#"{{"id":"manual-1","eventTypeId":"et1","startDateTime":"{}","endDateTime":"{}"}}"#,
        manual_start.to_rfc3339(),
        (manual_start + Duration::minutes(30)).to_rfc3339()
    );
    let raw = common::send(app.clone(), &common::post_request("/slots", &manual_body)).await;
    assert!(
        raw.contains("HTTP/1.1 200"),
        "ручной слот опубликован: {raw}"
    );

    // Гость занимает завтрашний слот 10:30.
    let booked = state
        .slots
        .list()
        .into_iter()
        .find(|slot| {
            slot.event_type_id == "et1"
                && slot.start_date_time > Utc::now()
                && slot.start_date_time.format("%H:%M").to_string() == "10:30"
        })
        .expect("слот 10:30materialизован");
    let booking_body = format!(
        r#"{{"id":"b1","slotId":"{}","guestName":"Гость","guestEmail":"g@example.com","createdAt":"{}"}}"#,
        booked.id,
        Utc::now().to_rfc3339()
    );
    let raw = common::send(
        app.clone(),
        &common::post_request("/bookings", &booking_body),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "запись создана: {raw}");

    // Сужение окна до 10:00–10:30: слоты 10:30 без записей удаляются.
    put_schedule(
        app.clone(),
        &schedule_body("UTC", &[1, 2, 3, 4, 5, 6, 7], "10:00", "10:30"),
    )
    .await;

    let slots = state.slots.list();
    let still_booked = slots.iter().find(|slot| slot.id == booked.id);
    assert!(
        still_booked.is_some(),
        "занятый слот не удаляется при смене расписания"
    );
    let late: Vec<_> = slots
        .iter()
        .filter(|slot| slot.event_type_id == "et1" && slot.id != booked.id)
        .filter(|slot| {
            slot.start_date_time
                .format("%H:%M")
                .to_string()
                .ends_with(":30")
                && slot.start_date_time.format("%H").to_string()
                    == booked.start_date_time.format("%H").to_string()
        })
        .collect();
    assert!(
        late.is_empty(),
        "незанятые слоты 10:30 вне нового окна удалены: {late:?}"
    );
    assert!(
        slots.iter().any(|slot| slot.id == "manual-1"),
        "ручной слот вне окон не затронут"
    );
}

#[tokio::test]
async fn empty_rules_disable_windows_but_keep_manual_and_booked() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());

    put_schedule(
        app.clone(),
        &schedule_body("UTC", &[1, 2, 3, 4, 5, 6, 7], "10:00", "11:00"),
    )
    .await;

    // Ручной слот вне окон + запись на сгенерированный слот.
    let manual_start = (Utc::now() + Duration::days(1))
        .date_naive()
        .and_hms_opt(20, 0, 0)
        .unwrap();
    let manual_start = Utc.from_local_datetime(&manual_start).single().unwrap();
    let manual_body = format!(
        r#"{{"id":"manual-1","eventTypeId":"et1","startDateTime":"{}","endDateTime":"{}"}}"#,
        manual_start.to_rfc3339(),
        (manual_start + Duration::minutes(30)).to_rfc3339()
    );
    let raw = common::send(app.clone(), &common::post_request("/slots", &manual_body)).await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let booked = state
        .slots
        .list()
        .into_iter()
        .find(|slot| slot.event_type_id == "et2")
        .expect("слот 60-минутного типа материализован");
    let booking_body = format!(
        r#"{{"id":"b1","slotId":"{}","guestName":"Гость","guestEmail":"g@example.com","createdAt":"{}"}}"#,
        booked.id,
        Utc::now().to_rfc3339()
    );
    let raw = common::send(
        app.clone(),
        &common::post_request("/bookings", &booking_body),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    // Пустое расписание выключает окна.
    put_schedule(app.clone(), &schedule_body("UTC", &[], "10:00", "17:00")).await;

    let slots = state.slots.list();
    assert_eq!(slots.len(), 2, "остались ручной и занятый слоты");
    assert!(slots.iter().any(|slot| slot.id == "manual-1"));
    assert!(slots.iter().any(|slot| slot.id == booked.id));
}

#[tokio::test]
async fn invalid_schedule_rejected_400_without_side_effects() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());

    let raw = put_schedule(
        app.clone(),
        &schedule_body("UTC", &[1, 2, 3, 4, 5], "10:00", "17:00"),
    )
    .await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");
    let mut before: Vec<_> = state.slots.list().iter().map(|s| s.id.clone()).collect();
    before.sort();

    let cases = [
        (
            "неизвестная зона",
            schedule_body("Mars/Olympus", &[1], "10:00", "17:00"),
        ),
        (
            "пустой набор дней",
            r#"{"timeZone":"UTC","rules":[{"weekdays":[],"startTime":"10:00","endTime":"17:00"}]}"#
                .to_string(),
        ),
        (
            "конец не позже начала",
            schedule_body("UTC", &[1], "17:00", "10:00"),
        ),
        (
            "время вне сетки",
            schedule_body("UTC", &[1], "10:15", "17:00"),
        ),
        (
            "день вне диапазона",
            schedule_body("UTC", &[8], "10:00", "17:00"),
        ),
    ];
    for (label, body) in cases {
        let raw = put_schedule(app.clone(), &body).await;
        assert!(
            raw.contains("HTTP/1.1 400"),
            "невалидное расписание ({label}) должно давать 400, got: {raw}"
        );
    }

    let mut after: Vec<_> = state.slots.list().iter().map(|s| s.id.clone()).collect();
    after.sort();
    assert_eq!(before, after, "валидация до мутации: слоты не изменились");
}

#[tokio::test]
async fn manual_slot_outside_schedule_survives_regeneration() {
    let state = seeded_state();
    let app = backend::app_with_state(state.clone());

    // Ручной слот вне будущих окон.
    let manual_start = (Utc::now() + Duration::days(1))
        .date_naive()
        .and_hms_opt(20, 0, 0)
        .unwrap();
    let manual_start = Utc.from_local_datetime(&manual_start).single().unwrap();
    let manual_body = format!(
        r#"{{"id":"manual-1","eventTypeId":"et1","startDateTime":"{}","endDateTime":"{}"}}"#,
        manual_start.to_rfc3339(),
        (manual_start + Duration::minutes(30)).to_rfc3339()
    );
    let raw = common::send(app.clone(), &common::post_request("/slots", &manual_body)).await;
    assert!(raw.contains("HTTP/1.1 200"), "got: {raw}");

    // Смена расписания — ручной слот вне окон не затрагивается.
    put_schedule(
        app.clone(),
        &schedule_body("Europe/Moscow", &[1, 2, 3, 4, 5], "10:00", "17:00"),
    )
    .await;

    let slots = state.slots.list();
    assert!(
        slots.iter().any(|slot| slot.id == "manual-1"),
        "ручной слот пережил перегенерацию"
    );
    // Окна в Europe/Moscow (UTC+3) дают слоты 07:00 UTC — конвертация
    // настенного времени выполнена по зоне расписания.
    let first = sorted_starts(&slots, "et1")
        .into_iter()
        .find(|start| start.format("%H:%M").to_string() == "07:00");
    assert!(first.is_some(), "10:00 MSK = 07:00 UTC");
}
