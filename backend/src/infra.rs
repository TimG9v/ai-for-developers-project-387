//! Инфраструктура: in-memory хранилища типов встреч и слотов.
//! Данные не переживают перезапуск сервера (решение спеки — до введения БД).

use std::sync::Mutex;

use crate::api::api_types::{Booking, EventType, Slot};
use crate::domain::{BookingsRepository, EventTypesRepository, RescheduleError, SlotsRepository};

#[derive(Default)]
pub struct InMemoryEventTypes {
    items: Mutex<Vec<EventType>>,
}

impl InMemoryEventTypes {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventTypesRepository for InMemoryEventTypes {
    fn list(&self) -> Vec<EventType> {
        self.items.lock().expect("event types lock").clone()
    }

    fn get(&self, id: &str) -> Option<EventType> {
        self.items
            .lock()
            .expect("event types lock")
            .iter()
            .find(|event_type| event_type.id == id)
            .cloned()
    }

    fn add(&self, event_type: EventType) {
        self.items
            .lock()
            .expect("event types lock")
            .push(event_type);
    }
}

#[derive(Default)]
pub struct InMemorySlots {
    items: Mutex<Vec<Slot>>,
}

impl InMemorySlots {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SlotsRepository for InMemorySlots {
    fn list(&self) -> Vec<Slot> {
        self.items.lock().expect("slots lock").clone()
    }

    fn get(&self, id: &str) -> Option<Slot> {
        self.items
            .lock()
            .expect("slots lock")
            .iter()
            .find(|slot| slot.id == id)
            .cloned()
    }

    fn add(&self, slot: Slot) {
        self.items.lock().expect("slots lock").push(slot);
    }
}

#[derive(Default)]
pub struct InMemoryBookings {
    items: Mutex<Vec<Booking>>,
}

impl InMemoryBookings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BookingsRepository for InMemoryBookings {
    fn try_add(&self, booking: Booking) -> bool {
        // Проверка занятости и вставка — под одной блокировкой:
        // два одновременных запроса не создадут вторую запись.
        let mut items = self.items.lock().expect("bookings lock");
        if items
            .iter()
            .any(|existing| existing.slot_id == booking.slot_id)
        {
            return false;
        }
        items.push(booking);
        true
    }

    fn contains_slot(&self, slot_id: &str) -> bool {
        self.items
            .lock()
            .expect("bookings lock")
            .iter()
            .any(|booking| booking.slot_id == slot_id)
    }

    fn get(&self, id: &str) -> Option<Booking> {
        self.items
            .lock()
            .expect("bookings lock")
            .iter()
            .find(|booking| booking.id == id)
            .cloned()
    }

    fn list(&self) -> Vec<Booking> {
        self.items.lock().expect("bookings lock").clone()
    }

    fn remove(&self, id: &str) -> bool {
        let mut items = self.items.lock().expect("bookings lock");
        let before = items.len();
        items.retain(|booking| booking.id != id);
        items.len() < before
    }

    fn reschedule(&self, booking_id: &str, new_slot_id: &str) -> Result<Booking, RescheduleError> {
        // Занятость нового слота и смена слота — под одной блокировкой:
        // два одновременных переноса (или перенос и запись) не устроят
        // вторую запись на слот.
        let mut items = self.items.lock().expect("bookings lock");
        if items.iter().any(|existing| existing.slot_id == new_slot_id) {
            return Err(RescheduleError::NewSlotTaken);
        }
        let booking = items
            .iter_mut()
            .find(|booking| booking.id == booking_id)
            .ok_or(RescheduleError::BookingNotFound)?;
        booking.slot_id = new_slot_id.to_string();
        Ok(booking.clone())
    }
}
