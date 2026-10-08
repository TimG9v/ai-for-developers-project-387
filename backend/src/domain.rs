//! Слой приложения: контракты хранения и правила домена.

use chrono::{DateTime, Datelike, Duration, LocalResult, Utc};
use chrono_tz::Tz;

use crate::api::api_types::{Booking, EventType, Slot, WorkingHours};

/// Окно записи — константа от текущей даты (решение спеки; не настраивается).
pub const BOOKING_WINDOW_DAYS: i64 = 14;

/// Репозиторий типов встреч; имплементации живут в инфраструктуре.
pub trait EventTypesRepository: Send + Sync {
    fn list(&self) -> Vec<EventType>;

    fn get(&self, id: &str) -> Option<EventType>;

    /// Атомарный insert-if-absent по содержимому: тип с тем же названием
    /// (пробелы по краям не различаются) и длительностью уже существует —
    /// повторные отправки формы и сетевые повторы не создают дубликатов.
    /// Возвращает false, если такой тип уже есть.
    fn try_add(&self, event_type: EventType) -> bool;
}

/// Причины отклонения типа встречи сервером.
#[derive(Debug)]
pub enum EventTypeValidationError {
    EmptyTitle,
    NonPositiveDuration,
}

/// Серверная валидация: название не пустое (пробелы по краям не считаются),
/// длительность положительная.
pub fn validate_event_type(event_type: &EventType) -> Result<(), EventTypeValidationError> {
    if event_type.title.trim().is_empty() {
        return Err(EventTypeValidationError::EmptyTitle);
    }
    if event_type.duration_minutes <= 0 {
        return Err(EventTypeValidationError::NonPositiveDuration);
    }
    Ok(())
}

/// Репозиторий слотов; имплементации живут в инфраструктуре.
pub trait SlotsRepository: Send + Sync {
    fn list(&self) -> Vec<Slot>;

    fn get(&self, id: &str) -> Option<Slot>;

    /// Атомарный insert-if-absent по содержимому: слот того же типа встречи
    /// с тем же началом (конец детерминирован длительностью типа) уже
    /// существует — повторные отправки формы и сетевые повторы не создают
    /// дубликатов. Возвращает false, если такой слот уже есть.
    fn try_add(&self, slot: Slot) -> bool;

    /// Удалить слот по идентификатору (перегенерация рабочих часов);
    /// true — слот был и удалён.
    fn remove(&self, id: &str) -> bool;
}

/// Репозиторий записей; имплементации живут в инфраструктуре.
pub trait BookingsRepository: Send + Sync {
    /// Атомарный insert-if-absent: записи не пересекаются по интервалу
    /// времени слота — ни дважды на одном слоте, ни между слотами разных
    /// типов встреч (ADR 0004). Возвращает false, если интервал занят.
    fn try_add(&self, booking: Booking, slot: &Slot) -> bool;

    /// Занят ли слот какой-либо записью.
    fn contains_slot(&self, slot_id: &str) -> bool;

    /// Занят ли интервал времени какой-либо записью (ADR 0004): true —
    /// пересекается с интервалом хотя бы одной записи.
    fn is_interval_taken(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool;

    /// Запись по идентификатору (ссылка управления записью).
    fn get(&self, id: &str) -> Option<Booking>;

    /// Все записи — ракурс владельца (история 4).
    fn list(&self) -> Vec<Booking>;

    /// Удалить запись по id; true — запись была и отменена.
    fn remove(&self, id: &str) -> bool;

    /// Атомарный перенос записи на новый слот: занятость интервала нового
    /// слота и смена слота — под одной блокировкой (по образцу try_add).
    fn reschedule(&self, booking_id: &str, new_slot: &Slot) -> Result<Booking, RescheduleError>;
}

/// Слот виден в календаре записи, только если начинается в окне 14 дней:
/// не в прошлом и не позже 14-го дня включительно.
pub fn is_within_booking_window(start: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    start >= now && start <= now + Duration::days(BOOKING_WINDOW_DAYS)
}

/// Шаг сетки начала слота — 30 минут (обязательное требование проекта).
const SLOT_GRID_SECONDS: i64 = 30 * 60;

/// Начало слота на 30-минутной сетке: …:00 / …:30, секунды и доли — ноль.
/// Суб-секунды проверяются отдельно: timestamp() их отбрасывает.
pub fn is_on_grid(start: DateTime<Utc>) -> bool {
    start.timestamp_subsec_nanos() == 0 && start.timestamp().rem_euclid(SLOT_GRID_SECONDS) == 0
}

/// Причины отклонения слота сервером.
#[derive(Debug)]
pub enum SlotValidationError {
    EndBeforeStart,
    OutsideBookingWindow,
    DurationMismatch,
    OffGridStart,
}

/// Серверная валидация слота: интервал непустой, начало в окне 14 дней,
/// длительность интервала равна длительности типа встречи
/// (словарь: «длительность слота определяется его типом встречи»),
/// начало на 30-минутной сетке.
pub fn validate_slot(
    slot: &Slot,
    now: DateTime<Utc>,
    event_type: &EventType,
) -> Result<(), SlotValidationError> {
    if slot.end_date_time <= slot.start_date_time {
        return Err(SlotValidationError::EndBeforeStart);
    }
    if !is_within_booking_window(slot.start_date_time, now) {
        return Err(SlotValidationError::OutsideBookingWindow);
    }
    let actual_minutes = (slot.end_date_time - slot.start_date_time).num_minutes();
    if actual_minutes != i64::from(event_type.duration_minutes) {
        return Err(SlotValidationError::DurationMismatch);
    }
    if !is_on_grid(slot.start_date_time) {
        return Err(SlotValidationError::OffGridStart);
    }
    Ok(())
}

/// Причины отклонения записи гостя сервером.
#[derive(Debug)]
pub enum BookingValidationError {
    EmptyGuestName,
    EmptyGuestEmail,
    SlotOutsideWindow,
}

/// Серверная валидация записи: имя и email гостя обязательны (история 13),
/// слот записи — в окне 14 дней (история 16: правило едино для всех).
/// Занятость слота проверяется отдельно и атомарно (try_add).
pub fn validate_booking(
    booking: &Booking,
    slot: &Slot,
    now: DateTime<Utc>,
) -> Result<(), BookingValidationError> {
    if booking.guest_name.trim().is_empty() {
        return Err(BookingValidationError::EmptyGuestName);
    }
    if booking.guest_email.trim().is_empty() {
        return Err(BookingValidationError::EmptyGuestEmail);
    }
    if !is_within_booking_window(slot.start_date_time, now) {
        return Err(BookingValidationError::SlotOutsideWindow);
    }
    Ok(())
}

/// Причины отклонения переноса записи сервером.
#[derive(Debug, PartialEq)]
pub enum RescheduleError {
    BookingNotFound,
    NewSlotNotFound,
    EventTypeMismatch,
    NewSlotOutsideWindow,
    NewSlotOffGrid,
    NewSlotTaken,
}

/// Серверная валидация переноса: новый слот — того же типа встречи
/// (длительность слота определяется типом), в окне 14 дней и на
/// 30-минутной сетке. Существование записи и нового слота проверяет роутер;
/// занятость нового слота — отдельно и атомарно
/// (BookingsRepository::reschedule).
pub fn validate_reschedule(
    old_slot: &Slot,
    new_slot: &Slot,
    now: DateTime<Utc>,
) -> Result<(), RescheduleError> {
    if old_slot.event_type_id != new_slot.event_type_id {
        return Err(RescheduleError::EventTypeMismatch);
    }
    if !is_within_booking_window(new_slot.start_date_time, now) {
        return Err(RescheduleError::NewSlotOutsideWindow);
    }
    if !is_on_grid(new_slot.start_date_time) {
        return Err(RescheduleError::NewSlotOffGrid);
    }
    Ok(())
}

/// Репозиторий единственного расписания рабочих часов владельца
/// (расписание одно на владельца — решение репортёра); имплементации
/// живут в инфраструктуре.
pub trait WorkingHoursRepository: Send + Sync {
    /// Текущее расписание; по умолчанию — пустое (окна выключены).
    fn get(&self) -> WorkingHours;

    /// Сохранить расписание. Валидация — забота домена до вызова.
    fn set(&self, working_hours: WorkingHours);
}

/// Разбор настенного времени «HH:MM» → (часы, минуты).
fn parse_wall_time(value: &str) -> Option<(u32, u32)> {
    let (hours, minutes) = value.split_once(':')?;
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some((hours, minutes))
}

/// Причины отклонения расписания рабочих часов сервером.
#[derive(Debug, PartialEq)]
pub enum WorkingHoursValidationError {
    UnknownTimeZone,
    EmptyWeekdays,
    WeekdayOutOfRange,
    BadTimeFormat,
    OffGridTime,
    EndNotAfterStart,
}

/// Валидация расписания (до любой мутации): зона — известный IANA; в каждом
/// правиле дни недели непусты и в диапазоне 1..=7 (ISO: 1 — понедельник),
/// времена «HH:MM» на 30-минутной сетке, конец строго позже начала
/// (окно — в пределах одних суток). Пустой список правил валиден:
/// расписание без окон выключает рабочие окна. Возвращает разобранную
/// зону для развёртывания.
pub fn validate_working_hours(
    working_hours: &WorkingHours,
) -> Result<Tz, WorkingHoursValidationError> {
    let Ok(time_zone) = working_hours.time_zone.parse::<Tz>() else {
        return Err(WorkingHoursValidationError::UnknownTimeZone);
    };
    for rule in &working_hours.rules {
        if rule.weekdays.is_empty() {
            return Err(WorkingHoursValidationError::EmptyWeekdays);
        }
        if rule.weekdays.iter().any(|day| !(1..=7).contains(day)) {
            return Err(WorkingHoursValidationError::WeekdayOutOfRange);
        }
        let Some((start_hours, start_minutes)) = parse_wall_time(&rule.start_time) else {
            return Err(WorkingHoursValidationError::BadTimeFormat);
        };
        let Some((end_hours, end_minutes)) = parse_wall_time(&rule.end_time) else {
            return Err(WorkingHoursValidationError::BadTimeFormat);
        };
        if start_minutes % 30 != 0 || end_minutes % 30 != 0 {
            return Err(WorkingHoursValidationError::OffGridTime);
        }
        if (end_hours, end_minutes) <= (start_hours, start_minutes) {
            return Err(WorkingHoursValidationError::EndNotAfterStart);
        }
    }
    Ok(time_zone)
}

/// Развёртывание расписания в конкретные слоты: «расписание × типы встреч»
/// на горизонте окна записи (те же 14 дней, что в валидации слота).
/// Настенное время конвертируется в UTC по дням (DST-устойчиво): «10:00
/// в зоне владельца» — корректный момент своего дня, а не фиксированное
/// смещение; несуществующее настенное время (щель перевода часов)
/// пропускается. Слоты кладутся встык от начала до конца окна; каждый
/// сгенерированный слот проходит инварианты валидации слота: начало в окне
/// записи, 30-минутная сетка (типы с длительностью не кратной 30 минутам
/// окнами не публикуются — они остаются ручному сценарию), длительность
/// равна длительности типа.
pub fn deploy_working_hours(
    working_hours: &WorkingHours,
    event_types: &[EventType],
    now: DateTime<Utc>,
) -> Vec<Slot> {
    let Ok(time_zone) = working_hours.time_zone.parse::<Tz>() else {
        return Vec::new();
    };
    let mut slots = Vec::new();
    for day_offset in 0..=BOOKING_WINDOW_DAYS {
        let day = now.with_timezone(&time_zone).date_naive() + Duration::days(day_offset);
        let weekday = day.weekday().number_from_monday();
        for rule in &working_hours.rules {
            if !rule.weekdays.contains(&(weekday as i32)) {
                continue;
            }
            let Some((start_hours, start_minutes)) = parse_wall_time(&rule.start_time) else {
                continue;
            };
            let Some((end_hours, end_minutes)) = parse_wall_time(&rule.end_time) else {
                continue;
            };
            let Some(window_start_local) = day.and_hms_opt(start_hours, start_minutes, 0) else {
                continue;
            };
            let Some(window_end_local) = day.and_hms_opt(end_hours, end_minutes, 0) else {
                continue;
            };
            // Перевод часов: неоднозначное настенное время берём первым
            // вхождением, несуществующее (щель) — пропускаем день.
            let window_start = match window_start_local.and_local_timezone(time_zone) {
                LocalResult::Single(at) => at,
                LocalResult::Ambiguous(earliest, _) => earliest,
                LocalResult::None => continue,
            };
            let window_end = match window_end_local.and_local_timezone(time_zone) {
                LocalResult::Single(at) => at,
                LocalResult::Ambiguous(earliest, _) => earliest,
                LocalResult::None => continue,
            };
            let window = window_end - window_start;
            for event_type in event_types {
                let duration_minutes = i64::from(event_type.duration_minutes);
                if duration_minutes <= 0
                    || duration_minutes % 30 != 0
                    || duration_minutes > window.num_minutes()
                {
                    continue;
                }
                let duration = Duration::minutes(duration_minutes);
                let mut cursor = window_start;
                while cursor + duration <= window_end {
                    let start = cursor.with_timezone(&Utc);
                    // Инварианты слота: окно записи и сетка (зоны со
                    // смещением не кратным четверти часа дают вне-сеточный
                    // UTC — такие дни не публикуются окнами).
                    if !is_within_booking_window(start, now) || !is_on_grid(start) {
                        cursor += duration;
                        continue;
                    }
                    slots.push(Slot {
                        // Детерминированный id: повторный PUT того же
                        // расписания получает те же ключи содержимого.
                        id: format!("wh-{}-{}", event_type.id, start.timestamp()),
                        event_type_id: event_type.id.clone(),
                        start_date_time: start,
                        end_date_time: (cursor + duration).with_timezone(&Utc),
                    });
                    cursor += duration;
                }
            }
        }
    }
    slots
}

#[cfg(test)]
mod working_hours_tests {
    use chrono::TimeZone;

    use super::*;
    use crate::api::api_types::WorkingHoursRule;

    fn event_type(id: &str, duration_minutes: i32) -> EventType {
        EventType {
            id: id.to_string(),
            title: "Созвон".to_string(),
            description: None,
            duration_minutes,
        }
    }

    fn schedule(time_zone: &str, weekdays: Vec<i32>) -> WorkingHours {
        WorkingHours {
            time_zone: time_zone.to_string(),
            rules: vec![WorkingHoursRule {
                weekdays,
                start_time: "10:00".to_string(),
                end_time: "10:30".to_string(),
            }],
        }
    }

    #[test]
    fn deployment_converts_wall_time_per_day_across_dst() {
        // Europe/Berlin: 25 октября 2026 — переход на зимнее время
        // (CEST UTC+2 → CET UTC+1). Вторник в правилах; горизонт от
        // вторника 20 октября накрывает вторники 20.10 и 27.10.
        let now = Utc.with_ymd_and_hms(2026, 10, 20, 6, 0, 0).unwrap();
        let slots = deploy_working_hours(
            &schedule("Europe/Berlin", vec![2]),
            &[event_type("et1", 30)],
            now,
        );
        let mut starts: Vec<DateTime<Utc>> = slots.iter().map(|s| s.start_date_time).collect();
        starts.sort();
        assert_eq!(starts.len(), 2, "два вторника в горизонте: {starts:?}");
        assert_eq!(
            starts[0],
            Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap(),
            "10:00 CEST = 08:00 UTC"
        );
        assert_eq!(
            starts[1],
            Utc.with_ymd_and_hms(2026, 10, 27, 9, 0, 0).unwrap(),
            "10:00 CET = 09:00 UTC — конвертация по дням, не смещением"
        );
    }

    #[test]
    fn deployment_lays_slots_back_to_back_and_on_grid() {
        // Окно 10:00–12:00 UTC, тип 30 минут: 10:00, 10:30, 11:00, 11:30
        // в каждый попадающий в горизонт вторник (27.10 и 03.11).
        let now = Utc.with_ymd_and_hms(2026, 10, 21, 0, 0, 0).unwrap();
        let mut working_hours = schedule("UTC", vec![2]);
        working_hours.rules[0].end_time = "12:00".to_string();
        let slots = deploy_working_hours(&working_hours, &[event_type("et1", 30)], now);
        let mut starts: Vec<DateTime<Utc>> = slots.iter().map(|s| s.start_date_time).collect();
        starts.sort();
        assert_eq!(starts.len(), 8, "два вторника × 4 слота: {starts:?}");
        for pair in starts.windows(2) {
            if pair[0].date_naive() == pair[1].date_naive() {
                assert_eq!(pair[1] - pair[0], Duration::minutes(30), "встык");
            }
            assert!(is_on_grid(pair[0]), "сетка 30 минут");
        }
    }

    #[test]
    fn deployment_skips_types_off_grid_duration() {
        // Тип с длительностью 20 минут не разворачивается окнами (инвариант
        // сетки), тип 30 минут — разворачивается.
        let now = Utc.with_ymd_and_hms(2026, 10, 20, 0, 0, 0).unwrap();
        let slots = deploy_working_hours(
            &schedule("UTC", vec![2]),
            &[event_type("et20", 20), event_type("et30", 30)],
            now,
        );
        assert!(slots.iter().all(|s| s.event_type_id == "et30"));
    }

    #[test]
    fn validation_rejects_unknown_zone_and_bad_rules() {
        let cases: Vec<(WorkingHours, WorkingHoursValidationError)> = vec![
            (
                schedule("Mars/Olympus", vec![1]),
                WorkingHoursValidationError::UnknownTimeZone,
            ),
            (
                schedule("UTC", vec![]),
                WorkingHoursValidationError::EmptyWeekdays,
            ),
            (
                schedule("UTC", vec![0]),
                WorkingHoursValidationError::WeekdayOutOfRange,
            ),
            (
                schedule("UTC", vec![8]),
                WorkingHoursValidationError::WeekdayOutOfRange,
            ),
        ];
        for (working_hours, expected) in cases {
            assert_eq!(
                validate_working_hours(&working_hours),
                Err(expected),
                "{working_hours:?}"
            );
        }
        let mut off_grid = schedule("UTC", vec![1]);
        off_grid.rules[0].start_time = "10:15".to_string();
        assert_eq!(
            validate_working_hours(&off_grid),
            Err(WorkingHoursValidationError::OffGridTime)
        );
        let mut reversed = schedule("UTC", vec![1]);
        reversed.rules[0].start_time = "17:00".to_string();
        reversed.rules[0].end_time = "10:00".to_string();
        assert_eq!(
            validate_working_hours(&reversed),
            Err(WorkingHoursValidationError::EndNotAfterStart)
        );
        let mut bad_format = schedule("UTC", vec![1]);
        bad_format.rules[0].start_time = "1000".to_string();
        assert_eq!(
            validate_working_hours(&bad_format),
            Err(WorkingHoursValidationError::BadTimeFormat)
        );
    }

    #[test]
    fn validation_accepts_empty_rules_and_valid_schedule() {
        let empty = WorkingHours {
            time_zone: "Europe/Moscow".to_string(),
            rules: Vec::new(),
        };
        assert_eq!(
            validate_working_hours(&empty),
            Ok("Europe/Moscow".parse().unwrap())
        );
        assert_eq!(
            validate_working_hours(&schedule("UTC", vec![1, 2, 5])),
            Ok("UTC".parse().unwrap())
        );
    }
}
