"use client";

import { useState, type FormEvent } from "react";

import { bookingsCreate, type Slot } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

import { formatSlotInterval } from "@/lib/slot-time";

type BookingFormError = "empty-fields";

const ERROR_MESSAGES: Record<BookingFormError, string> = {
  "empty-fields": "Укажите имя и email",
};

/**
 * Форма записи гостя на выбранный слот: имя и email обязательны (проверяются
 * формой и сервером). Успех и отказ сервера (409 занятость — история 11,
 * 404 устаревшая ссылка — история 12) обрабатывает родитель: календарь
 * перезагружается, сообщения переживают обновление списка.
 */
export function BookingForm({
  slot,
  onBooked,
  onRejected,
}: {
  slot: Slot;
  /** Успех: родитель показывает подтверждение и обновляет календарь. */
  onBooked: (guestEmail: string) => void;
  /** Отказ сервера: родитель показывает сообщение и обновляет календарь. */
  onRejected: (status: number | undefined) => void;
}) {
  const [guestName, setGuestName] = useState("");
  const [guestEmail, setGuestEmail] = useState("");
  const [error, setError] = useState<BookingFormError | null>(null);

  const interval = formatSlotInterval(
    new Date(slot.startDateTime),
    new Date(slot.endDateTime),
  );

  async function handleSubmit(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault();
    if (guestName.trim() === "" || guestEmail.trim() === "") {
      setError("empty-fields");
      return;
    }

    const { error: createError } = await bookingsCreate({
      body: {
        id: crypto.randomUUID(),
        slotId: slot.id,
        guestName,
        guestEmail,
        createdAt: new Date().toISOString(),
      },
    });
    if (createError !== undefined) {
      onRejected((createError as { status?: number }).status);
      return;
    }

    // Подтверждение рендерит родитель: после перезагрузки календаря
    // выбранный слот уходит, форма размонтируется.
    onBooked(guestEmail);
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="flex w-full flex-col gap-4 rounded-xl border bg-card p-6 text-card-foreground"
    >
      <h3 className="text-xl font-semibold">Запись на встречу</h3>
      <p className="text-sm text-muted-foreground">{interval}</p>
      <div className="flex flex-col gap-2">
        <label htmlFor="booking-name" className="text-sm font-medium">
          Имя
        </label>
        <input
          id="booking-name"
          value={guestName}
          onChange={(changeEvent) => setGuestName(changeEvent.target.value)}
          className="rounded-lg border bg-background px-3 py-2"
        />
      </div>
      <div className="flex flex-col gap-2">
        <label htmlFor="booking-email" className="text-sm font-medium">
          Email
        </label>
        <input
          id="booking-email"
          type="email"
          value={guestEmail}
          onChange={(changeEvent) => setGuestEmail(changeEvent.target.value)}
          className="rounded-lg border bg-background px-3 py-2"
        />
      </div>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {ERROR_MESSAGES[error]}
        </p>
      )}
      <button
        type="submit"
        className="rounded-lg bg-primary px-4 py-2 text-primary-foreground"
      >
        Записаться
      </button>
    </form>
  );
}
