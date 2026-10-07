//! Демо-данные in-memory хранилища: публикуются при старте бинарника,
//! чтобы опубликованный сценарий был проверяем после каждого перезапуска
//! (хранилище не переживает рестарт — решение спеки до введения БД).
//! Отключается переменной окружения `BACKEND_SEED_DEMO=0`.

use chrono::{Duration, TimeZone, Utc};

use crate::api::api_types::{EventType, Slot};
use crate::domain::{EventTypesRepository, SlotsRepository};

/// Демо-типы встреч: id, название, длительность (мин).
const DEMO_EVENT_TYPES: [(&str, &str, i32); 2] = [
    ("et-demo-intro", "Знакомство (демо)", 30),
    ("et-demo-call", "Разбор (демо)", 60),
];

/// Час публикации (UTC) демо-слота для каждого типа. Разные часы — чтобы
/// демо-слоты не пересекались по времени (инвариант ADR 0004).
const DEMO_HOURS: [(&str, u32); 2] = [("et-demo-intro", 10), ("et-demo-call", 15)];

/// Сколько дней вперёд публиковать демо-слоты (окно записи — 14 дней).
const DEMO_DAYS_AHEAD: i64 = 3;

/// Наполнить хранилища демо-данными. Идемпотентно: если типы встреч уже
/// есть (в т.ч. от повторного вызова), ничего не добавляет.
pub fn seed_demo(event_types: &dyn EventTypesRepository, slots: &dyn SlotsRepository) {
    if !event_types.list().is_empty() {
        return;
    }
    for (id, title, duration_minutes) in DEMO_EVENT_TYPES {
        event_types.add(EventType {
            id: id.to_string(),
            title: title.to_string(),
            description: Some("Демо-тип встречи".to_string()),
            duration_minutes,
        });
    }
    for day in 1..=DEMO_DAYS_AHEAD {
        for (type_id, hour) in DEMO_HOURS {
            let date = (Utc::now() + Duration::days(day)).date_naive();
            let Some(naive) = date.and_hms_opt(hour, 0, 0) else {
                continue;
            };
            // Перевод в UTC без переходов на летнее время: single() всегда
            // определён для даты, собранной из UTC-календаря.
            let Some(start) = Utc.from_local_datetime(&naive).single() else {
                continue;
            };
            let duration = event_types
                .get(type_id)
                .expect("тип добавлен выше")
                .duration_minutes;
            slots.add(Slot {
                id: format!("s-demo-{type_id}-{day}"),
                event_type_id: type_id.to_string(),
                start_date_time: start,
                end_date_time: start + Duration::minutes(i64::from(duration)),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::is_on_grid;
    use crate::infra::{InMemoryEventTypes, InMemorySlots};

    #[test]
    fn seed_demo_publishes_types_and_on_grid_slots_in_window() {
        let event_types = InMemoryEventTypes::new();
        let slots = InMemorySlots::new();

        seed_demo(&event_types, &slots);

        assert_eq!(event_types.list().len(), 2);
        let now = Utc::now();
        let published = slots.list();
        assert_eq!(published.len(), 6, "3 дня × 2 типа");
        for slot in &published {
            assert!(is_on_grid(slot.start_date_time), "сетка 30 мин");
            assert!(slot.start_date_time > now, "только будущее");
            assert!(
                slot.start_date_time <= now + Duration::days(14),
                "окно 14 дней"
            );
        }
        // Демо-слоты не пересекаются попарно (ADR 0004): гость может
        // забронировать любой предложенный, не упираясь в 409.
        for (i, a) in published.iter().enumerate() {
            for b in published.iter().skip(i + 1) {
                assert!(
                    a.end_date_time <= b.start_date_time || b.end_date_time <= a.start_date_time,
                    "демо-слоты пересекаются: {a:?} и {b:?}"
                );
            }
        }
    }

    #[test]
    fn seed_demo_is_idempotent() {
        let event_types = InMemoryEventTypes::new();
        let slots = InMemorySlots::new();

        seed_demo(&event_types, &slots);
        seed_demo(&event_types, &slots);

        assert_eq!(event_types.list().len(), 2);
        assert_eq!(slots.list().len(), 6);
    }
}
