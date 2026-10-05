pub mod api;
pub mod domain;
pub mod infra;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::api_types::{Booking, EventType, Slot};
use crate::domain::{BookingsRepository, EventTypesRepository, SlotsRepository};

/// Адрес для `TcpListener::bind`: loopback, порт из `BACKEND_PORT`
/// (дефолт 8081). `PORT` — не здесь: это публичный порт Next.js в контейнере.
pub fn bind_addr(backend_port: Option<&str>) -> String {
    format!("127.0.0.1:{}", backend_port.unwrap_or("8081"))
}

/// Состояние приложения: репозитории, с которыми собран роутер.
#[derive(Clone)]
pub struct AppState {
    pub event_types: Arc<dyn EventTypesRepository>,
    pub slots: Arc<dyn SlotsRepository>,
    pub bookings: Arc<dyn BookingsRepository>,
}

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
}

/// Роутер с дефолтным in-memory состоянием.
pub fn app() -> Router {
    app_with_state(AppState {
        event_types: Arc::new(infra::InMemoryEventTypes::new()),
        slots: Arc::new(infra::InMemorySlots::new()),
        bookings: Arc::new(infra::InMemoryBookings::new()),
    })
}

/// Роутер с подставленным состоянием (тесты собирают его сами).
pub fn app_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/event-types",
            get(list_event_types).post(create_event_type),
        )
        .route("/slots", get(list_slots).post(create_slot))
        .route("/bookings", get(list_bookings).post(create_booking))
        .route("/upcoming-meetings", get(list_upcoming_meetings))
        .with_state(state)
}

async fn health() -> Json<HealthStatus> {
    Json(HealthStatus { status: "ok" })
}

async fn list_event_types(State(state): State<AppState>) -> Json<Vec<EventType>> {
    Json(state.event_types.list())
}

/// Создание типа встречи владельцем. Невалидный ввод — контрактный 400:
/// и нераспарсиваемое тело, и нарушение правил домена.
async fn create_event_type(
    State(state): State<AppState>,
    event_type: Result<Json<EventType>, JsonRejection>,
) -> Response {
    let event_type = match event_type {
        Ok(Json(event_type)) => event_type,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    if domain::validate_event_type(&event_type).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    state.event_types.add(event_type.clone());
    Json(event_type).into_response()
}

#[derive(Deserialize)]
struct SlotsQuery {
    #[serde(rename = "eventTypeId")]
    event_type_id: Option<String>,
}

/// Календарь записи: свободные слоты выбранного типа в окне 14 дней;
/// занятые (с записью) не отдаются (история 14).
async fn list_slots(State(state): State<AppState>, query: Query<SlotsQuery>) -> Json<Vec<Slot>> {
    let now: DateTime<Utc> = Utc::now();
    let slots = state
        .slots
        .list()
        .into_iter()
        .filter(|slot| {
            query
                .event_type_id
                .as_deref()
                .is_none_or(|id| slot.event_type_id == id)
        })
        .filter(|slot| domain::is_within_booking_window(slot.start_date_time, now))
        .filter(|slot| !state.bookings.contains_slot(&slot.id))
        .collect();
    Json(slots)
}

/// Публикация слота владельцем. Несуществующий тип — контрактный 404,
/// слот вне окна или с пустым интервалом — контрактный 400.
async fn create_slot(
    State(state): State<AppState>,
    slot: Result<Json<Slot>, JsonRejection>,
) -> Response {
    let slot = match slot {
        Ok(Json(slot)) => slot,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let Some(event_type) = state.event_types.get(&slot.event_type_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if domain::validate_slot(&slot, Utc::now(), &event_type).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    state.slots.add(slot.clone());
    Json(slot).into_response()
}

/// Список записей — ракурс владельца (история 4); без охраны (ADR 0002).
async fn list_bookings(State(state): State<AppState>) -> Json<Vec<Booking>> {
    Json(state.bookings.list())
}

/// Предстоящие встречи владельца (истории 4–5): записи со временем слота
/// и типом встречи одним ответом; прошедшие слоты не отдаются.
async fn list_upcoming_meetings(
    State(state): State<AppState>,
) -> Json<Vec<api::api_types::UpcomingMeeting>> {
    let now: DateTime<Utc> = Utc::now();
    let meetings = state
        .bookings
        .list()
        .into_iter()
        .filter_map(|booking| {
            let slot = state.slots.get(&booking.slot_id)?;
            if slot.start_date_time < now {
                return None;
            }
            let event_type = state.event_types.get(&slot.event_type_id)?;
            Some(api::api_types::UpcomingMeeting {
                id: booking.id,
                slot_id: slot.id,
                guest_name: booking.guest_name,
                guest_email: booking.guest_email,
                start_date_time: slot.start_date_time,
                end_date_time: slot.end_date_time,
                event_type_id: event_type.id,
                event_title: event_type.title,
            })
        })
        .collect();
    Json(meetings)
}

/// Запись гостя на слот. Несуществующий слот — контрактный 404, пустое
/// имя/email или слот вне окна — контрактный 400, занятый слот — 409:
/// проверка занятости и вставка атомарны (insert-if-absent).
async fn create_booking(
    State(state): State<AppState>,
    booking: Result<Json<Booking>, JsonRejection>,
) -> Response {
    let booking = match booking {
        Ok(Json(booking)) => booking,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let Some(slot) = state.slots.get(&booking.slot_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if domain::validate_booking(&booking, &slot, Utc::now()).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if !state.bookings.try_add(booking.clone()) {
        return StatusCode::CONFLICT.into_response();
    }
    Json(booking).into_response()
}

#[cfg(test)]
mod bind_addr_tests {
    use super::bind_addr;

    #[test]
    fn default_is_loopback_8081() {
        assert_eq!(bind_addr(None), "127.0.0.1:8081");
    }

    #[test]
    fn backend_port_overrides_default() {
        assert_eq!(bind_addr(Some("9000")), "127.0.0.1:9000");
    }
}
