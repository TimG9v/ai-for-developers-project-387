"use client";

import { useCallback, useState } from "react";

import { slotsList, type EventType, type Slot } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

import { BookingForm } from "@/components/booking-form";
import { formatSlotInterval } from "@/lib/slot-time";

/**
 * Страница записи: гость выбирает тип встречи, затем свободный слот и
 * оформляет запись. Окно 14 дней и занятость отсекает сервер — клиент
 * рендерит только то, что вернул SDK.
 */
export function BookingEventTypes({
  initialEventTypes,
}: {
  initialEventTypes: EventType[];
}) {
  const [selectedEventTypeId, setSelectedEventTypeId] = useState<string | null>(
    null,
  );
  const [slots, setSlots] = useState<Slot[]>([]);
  const [selectedSlotId, setSelectedSlotId] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<{
    title: string;
    interval: string;
    email: string;
  } | null>(null);
  const [bookingError, setBookingError] = useState<string | null>(null);

  const selectedEventType = initialEventTypes.find(
    (eventType) => eventType.id === selectedEventTypeId,
  );
  const selectedSlot = slots.find((slot) => slot.id === selectedSlotId) ?? null;

  const chooseEventType = useCallback(async (eventTypeId: string) => {
    setSelectedEventTypeId(eventTypeId);
    setSelectedSlotId(null);
    setConfirmation(null);
    setBookingError(null);
    const { data } = await slotsList({ query: { eventTypeId } });
    setSlots(data ?? []);
  }, []);

  const refreshCalendar = useCallback(async () => {
    if (selectedEventTypeId === null) {
      return;
    }
    // Занятый слот сервер больше не отдаёт — календарь обновляется.
    const result = await slotsList({ query: { eventTypeId: selectedEventTypeId } });
    setSlots(result?.data ?? []);
    setSelectedSlotId(null);
  }, [selectedEventTypeId]);

  const handleRejected = useCallback(
    (status: number | undefined) => {
      // История 11: понятное сообщение занятости; история 12: устаревшее
      // предложение. Календарь перезагружается — занятый слот исчезает.
      setBookingError(
        status === 409
          ? "Это время уже занято. Выберите другой слот"
          : status === 404
            ? "Этот слот больше не доступен — предложение устарело. Обновите страницу"
            : "Не удалось создать запись: сервер отклонил данные",
      );
      void refreshCalendar();
    },
    [refreshCalendar],
  );

  const handleBooked = useCallback(
    (guestEmail: string) => {
      setBookingError(null);
      const slot = slots.find((item) => item.id === selectedSlotId);
      if (slot) {
        setConfirmation({
          title: selectedEventType?.title ?? "встреча",
          interval: formatSlotInterval(
            new Date(slot.startDateTime),
            new Date(slot.endDateTime),
          ),
          email: guestEmail,
        });
      }
      void refreshCalendar();
    },
    [refreshCalendar, selectedEventType, slots, selectedSlotId],
  );

  if (initialEventTypes.length === 0) {
    return (
      <p className="text-muted-foreground">Пока нет доступных типов встреч</p>
    );
  }

  return (
    <div className="flex w-full max-w-2xl flex-col items-center gap-6">
      <ul className="flex w-full flex-col gap-4">
        {initialEventTypes.map((eventType) => (
          <li
            key={eventType.id}
            className="rounded-xl border bg-card p-6 text-card-foreground"
          >
            <div className="flex items-start justify-between gap-4">
              <div>
                <h2 className="text-xl font-semibold">{eventType.title}</h2>
                {eventType.description && (
                  <p className="text-muted-foreground">
                    {eventType.description}
                  </p>
                )}
                <p className="text-sm text-muted-foreground">
                  {eventType.durationMinutes} мин.
                </p>
              </div>
              <button
                type="button"
                onClick={() => void chooseEventType(eventType.id)}
                aria-label={`Выбрать ${eventType.title}`}
                className={`shrink-0 rounded-lg px-4 py-2 ${
                  eventType.id === selectedEventTypeId
                    ? "bg-primary text-primary-foreground"
                    : "border bg-background"
                }`}
              >
                {eventType.id === selectedEventTypeId ? "Выбран" : "Выбрать"}
              </button>
            </div>
          </li>
        ))}
      </ul>

      {selectedEventType === undefined ? (
        <p className="text-muted-foreground">
          Выберите тип встречи, чтобы увидеть свободные слоты
        </p>
      ) : slots.length === 0 ? (
        <p className="text-muted-foreground">
          У этого типа пока нет свободных слотов
        </p>
      ) : (
        <ul className="flex w-full flex-col gap-3" aria-label="Свободные слоты">
          {slots.map((slot) => {
            const interval = formatSlotInterval(
              new Date(slot.startDateTime),
              new Date(slot.endDateTime),
            );
            return (
              <li key={slot.id}>
                <button
                  type="button"
                  onClick={() => setSelectedSlotId(slot.id)}
                  aria-label={`Записаться на ${interval}`}
                  className={`w-full rounded-xl border p-4 text-left ${
                    slot.id === selectedSlotId
                      ? "border-primary bg-primary text-primary-foreground"
                      : "bg-card text-card-foreground"
                  }`}
                >
                  {interval}
                </button>
              </li>
            );
          })}
        </ul>
      )}

      {confirmation && (
        <div
          role="status"
          className="flex w-full flex-col gap-2 rounded-xl border bg-card p-6 text-card-foreground"
        >
          <h3 className="text-xl font-semibold">
            Вы записаны на «{confirmation.title}»
          </h3>
          <p className="text-muted-foreground">{confirmation.interval}</p>
          <p className="text-sm text-muted-foreground">
            Данные записи: {confirmation.email}
          </p>
        </div>
      )}

      {bookingError && (
        <p role="alert" className="text-destructive">
          {bookingError}
        </p>
      )}

      {selectedSlot && (
        <BookingForm
          slot={selectedSlot}
          onBooked={handleBooked}
          onRejected={handleRejected}
        />
      )}
    </div>
  );
}
