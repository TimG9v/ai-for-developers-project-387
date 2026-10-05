//! Инфраструктура: in-memory хранилища типов встреч и слотов.
//! Данные не переживают перезапуск сервера (решение спеки — до введения БД).

use std::sync::Mutex;

use crate::api::api_types::{Booking, EventType, Slot};
use crate::domain::{BookingsRepository, EventTypesRepository, SlotsRepository};

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

    fn list(&self) -> Vec<Booking> {
        self.items.lock().expect("bookings lock").clone()
    }
}
