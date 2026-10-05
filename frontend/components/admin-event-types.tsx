"use client";

import { useState, type FormEvent } from "react";

import { eventTypesCreate, eventTypesList, type EventType } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20): без этого импорта
// клиентский синглтон остаётся с пустым baseUrl — fetch уходит мимо /api.
import "@/src/api-config";

/**
 * Раздел владельца: форма создания типа встречи и список уже созданных.
 * Начальный список приходит из серверного компонента; после создания
 * список перезапрашивается через SDK.
 */
export function AdminEventTypes({
  initialEventTypes,
}: {
  initialEventTypes: EventType[];
}) {
  const [eventTypes, setEventTypes] = useState<EventType[]>(initialEventTypes);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [duration, setDuration] = useState("");
  const [formMessage, setFormMessage] = useState<string | null>(null);

  async function reloadEventTypes() {
    const { data } = await eventTypesList();
    setEventTypes(data ?? []);
  }

  async function handleSubmit(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault();
    const durationMinutes = Number(duration);
    if (
      title.trim() === "" ||
      !Number.isInteger(durationMinutes) ||
      durationMinutes <= 0
    ) {
      setFormMessage("Укажите название и длительность встречи");
      return;
    }

    const { error } = await eventTypesCreate({
      body: {
        id: crypto.randomUUID(),
        title: title.trim(),
        ...(description.trim() === ""
          ? {}
          : { description: description.trim() }),
        durationMinutes,
      },
    });
    if (error !== undefined) {
      setFormMessage("Не удалось создать тип встречи: сервер отклонил данные");
      return;
    }

    setFormMessage(null);
    setTitle("");
    setDescription("");
    setDuration("");
    await reloadEventTypes();
  }

  return (
    <>
      <form
        onSubmit={handleSubmit}
        className="flex w-full max-w-2xl flex-col gap-4 rounded-xl border bg-card p-6 text-card-foreground"
      >
        <div className="flex flex-col gap-2">
          <label htmlFor="event-type-title" className="text-sm font-medium">
            Название
          </label>
          <input
            id="event-type-title"
            value={title}
            onChange={(changeEvent) => setTitle(changeEvent.target.value)}
            className="rounded-lg border bg-background px-3 py-2"
          />
        </div>
        <div className="flex flex-col gap-2">
          <label
            htmlFor="event-type-description"
            className="text-sm font-medium"
          >
            Описание
          </label>
          <input
            id="event-type-description"
            value={description}
            onChange={(changeEvent) => setDescription(changeEvent.target.value)}
            className="rounded-lg border bg-background px-3 py-2"
          />
        </div>
        <div className="flex flex-col gap-2">
          <label
            htmlFor="event-type-duration"
            className="text-sm font-medium"
          >
            Длительность, минут
          </label>
          <input
            id="event-type-duration"
            type="number"
            value={duration}
            onChange={(changeEvent) => setDuration(changeEvent.target.value)}
            className="rounded-lg border bg-background px-3 py-2"
          />
        </div>
        {formMessage && (
          <p role="alert" className="text-sm text-destructive">
            {formMessage}
          </p>
        )}
        <button
          type="submit"
          className="rounded-lg bg-primary px-4 py-2 text-primary-foreground"
        >
          Создать тип встречи
        </button>
      </form>

      {eventTypes.length === 0 ? (
        <p className="text-muted-foreground">Пока нет типов встреч</p>
      ) : (
        <ul className="flex w-full max-w-2xl flex-col gap-4">
          {eventTypes.map((eventType) => (
            <li
              key={eventType.id}
              className="rounded-xl border bg-card p-6 text-card-foreground"
            >
              <h2 className="text-xl font-semibold">{eventType.title}</h2>
              {eventType.description && (
                <p className="text-muted-foreground">
                  {eventType.description}
                </p>
              )}
              <p className="text-sm text-muted-foreground">
                {eventType.durationMinutes} мин.
              </p>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}
