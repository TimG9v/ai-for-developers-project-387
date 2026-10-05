"use client";

import { useState, type FormEvent } from "react";

import { slotsCreate, type EventType } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

/**
 * Публикация слота владельцем: выбор типа встречи, дата и время начала.
 * Конец интервала вычисляется из длительности выбранного типа.
 * Правило окна 14 дней проверяет сервер — форма только публикует.
 */
export function AdminSlots({ eventTypes }: { eventTypes: EventType[] }) {
  const [eventTypeId, setEventTypeId] = useState("");
  const [date, setDate] = useState("");
  const [time, setTime] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [published, setPublished] = useState(false);

  async function handleSubmit(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault();
    if (eventTypeId === "" || date === "" || time === "") {
      setErrorMessage("Выберите тип встречи, дату и время");
      return;
    }

    const durationMinutes =
      eventTypes.find((eventType) => eventType.id === eventTypeId)
        ?.durationMinutes ?? 0;
    const startDateTime = new Date(`${date}T${time}`);
    const endDateTime = new Date(
      startDateTime.getTime() + durationMinutes * 60 * 1000,
    );

    const { error } = await slotsCreate({
      body: {
        id: crypto.randomUUID(),
        eventTypeId,
        startDateTime: startDateTime.toISOString(),
        endDateTime: endDateTime.toISOString(),
      },
    });
    if (error !== undefined) {
      setErrorMessage("Не удалось опубликовать слот: сервер отклонил данные");
      return;
    }

    setErrorMessage(null);
    setPublished(true);
    setDate("");
    setTime("");
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="flex w-full max-w-2xl flex-col gap-4 rounded-xl border bg-card p-6 text-card-foreground"
    >
      <h2 className="text-xl font-semibold">Публикация слота</h2>
      <div className="flex flex-col gap-2">
        <label htmlFor="slot-event-type" className="text-sm font-medium">
          Тип встречи
        </label>
        <select
          id="slot-event-type"
          value={eventTypeId}
          onChange={(changeEvent) => setEventTypeId(changeEvent.target.value)}
          className="rounded-lg border bg-background px-3 py-2"
        >
          <option value="">Выберите тип</option>
          {eventTypes.map((eventType) => (
            <option key={eventType.id} value={eventType.id}>
              {eventType.title}, {eventType.durationMinutes} мин.
            </option>
          ))}
        </select>
      </div>
      <div className="flex flex-col gap-2">
        <label htmlFor="slot-date" className="text-sm font-medium">
          Дата
        </label>
        <input
          id="slot-date"
          type="date"
          value={date}
          onChange={(changeEvent) => setDate(changeEvent.target.value)}
          className="rounded-lg border bg-background px-3 py-2"
        />
      </div>
      <div className="flex flex-col gap-2">
        <label htmlFor="slot-time" className="text-sm font-medium">
          Время
        </label>
        <input
          id="slot-time"
          type="time"
          value={time}
          onChange={(changeEvent) => setTime(changeEvent.target.value)}
          className="rounded-lg border bg-background px-3 py-2"
        />
      </div>
      {errorMessage && (
        <p role="alert" className="text-sm text-destructive">
          {errorMessage}
        </p>
      )}
      {published && <p className="text-sm text-muted-foreground">Слот опубликован</p>}
      <button
        type="submit"
        className="rounded-lg bg-primary px-4 py-2 text-primary-foreground"
      >
        Опубликовать слот
      </button>
    </form>
  );
}
