"use client";

import { useCallback, useEffect, useState } from "react";

import {
  bookingsCancel,
  bookingsReschedule,
  slotsList,
  upcomingMeetingsList,
  type Slot,
  type UpcomingMeeting,
} from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

import { formatSlotInterval } from "@/lib/slot-time";

/**
 * Состояние управления записью: гость пришёл по ссылке из подтверждения
 * (id записи — фактически unguessable-токен, АDR 0002). Записи нет в списке
 * предстоящих встреч и для незнакомого id, и для прошедшего времени —
 * гостю показываем одно и то же «Запись не найдена».
 */
type ManageView =
  | { kind: "loading" }
  | { kind: "missing" }
  | { kind: "cancelled" }
  | { kind: "meeting"; meeting: UpcomingMeeting }
  | { kind: "choosing-slot"; meeting: UpcomingMeeting; slots: Slot[] };

export function BookingManage({ bookingId }: { bookingId: string }) {
  // Пустой id — ссылка без параметра: записи нет, грузить нечего.
  const [view, setView] = useState<ManageView>(() =>
    bookingId === "" ? { kind: "missing" } : { kind: "loading" },
  );
  const [message, setMessage] = useState<string | null>(null);

  const findMeeting = useCallback(async (): Promise<UpcomingMeeting | null> => {
    const { data } = await upcomingMeetingsList();
    return (data ?? []).find((item) => item.id === bookingId) ?? null;
  }, [bookingId]);

  useEffect(() => {
    if (bookingId === "") {
      return;
    }
    let active = true;
    void findMeeting().then((meeting) => {
      if (active) {
        setView(
          meeting ? { kind: "meeting", meeting } : { kind: "missing" },
        );
      }
    });
    return () => {
      active = false;
    };
  }, [bookingId, findMeeting]);

  async function handleCancel() {
    setMessage(null);
    const { error } = await bookingsCancel({ path: { id: bookingId } });
    if (error === undefined) {
      setView({ kind: "cancelled" });
      return;
    }
    if ((error as { status?: number }).status === 404) {
      setView({ kind: "missing" });
      return;
    }
    setMessage("Не удалось отменить запись: сервер отклонил запрос");
  }

  async function openReschedule(meeting: UpcomingMeeting) {
    setMessage(null);
    // Перенос — только на слот того же типа встречи: длительность слота
    // определяется его типом (словарь).
    const { data } = await slotsList({
      query: { eventTypeId: meeting.eventTypeId },
    });
    const slots = data ?? [];
    if (slots.length === 0) {
      setMessage("Свободных слотов этого типа пока нет");
      return;
    }
    setView({ kind: "choosing-slot", meeting, slots });
  }

  async function handleReschedule(
    meeting: UpcomingMeeting,
    newSlotId: string,
  ) {
    setMessage(null);
    const { error } = await bookingsReschedule({
      path: { id: bookingId },
      body: { newSlotId },
    });
    if (error === undefined) {
      // Запись вернулась обновлённой; время берём из свежего списка встреч.
      const updated = await findMeeting();
      setView(
        updated ? { kind: "meeting", meeting: updated } : { kind: "missing" },
      );
      return;
    }
    const status = (error as { status?: number }).status;
    if (status === 409) {
      setMessage("Это время уже занято. Выберите другой слот");
      // Список слотов перезагружается: занятый целевой слот исчезает.
      const { data } = await slotsList({
        query: { eventTypeId: meeting.eventTypeId },
      });
      setView({ kind: "choosing-slot", meeting, slots: data ?? [] });
      return;
    }
    if (status === 404) {
      setView({ kind: "missing" });
      return;
    }
    setMessage("Не удалось перенести запись: сервер отклонил запрос");
  }

  if (view.kind === "loading") {
    return <p className="text-muted-foreground">Загружаем запись…</p>;
  }

  if (view.kind === "missing") {
    return (
      <div
        role="status"
        className="flex w-full flex-col gap-2 rounded-xl border bg-card p-6 text-card-foreground"
      >
        <p>Запись не найдена</p>
        <p className="text-sm text-muted-foreground">
          Проверьте ссылку из подтверждения записи — без авторизации это
          единственный способ найти вашу запись.
        </p>
        <a href="/booking" className="text-primary underline">
          На страницу записи
        </a>
      </div>
    );
  }

  if (view.kind === "cancelled") {
    return (
      <div
        role="status"
        className="flex w-full flex-col gap-2 rounded-xl border bg-card p-6 text-card-foreground"
      >
        <p>Запись отменена</p>
        <p className="text-sm text-muted-foreground">
          Слот снова свободен для других гостей.
        </p>
        <a href="/booking" className="text-primary underline">
          Записаться снова
        </a>
      </div>
    );
  }

  const meeting = view.meeting;
  const interval = formatSlotInterval(
    new Date(meeting.startDateTime),
    new Date(meeting.endDateTime),
  );

  return (
    <div className="flex w-full flex-col gap-4 rounded-xl border bg-card p-6 text-card-foreground">
      <h2 className="text-xl font-semibold">«{meeting.eventTitle}»</h2>
      <p className="text-sm text-muted-foreground">{interval}</p>
      <p className="text-sm text-muted-foreground">
        Гость: {meeting.guestName} · {meeting.guestEmail}
      </p>
      {message && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
      {view.kind === "meeting" ? (
        <div className="flex gap-3">
          <button
            type="button"
            onClick={() => void handleCancel()}
            className="rounded-lg border bg-background px-4 py-2"
          >
            Отменить запись
          </button>
          <button
            type="button"
            onClick={() => void openReschedule(view.meeting)}
            className="rounded-lg bg-primary px-4 py-2 text-primary-foreground"
          >
            Перенести запись
          </button>
        </div>
      ) : (
        <div className="flex w-full flex-col gap-2">
          <p className="text-sm font-medium">
            Выберите новый свободный слот того же типа встречи
          </p>
          {view.slots.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Свободных слотов этого типа пока нет
            </p>
          ) : (
            <ul className="flex w-full flex-col gap-2" aria-label="Свободные слоты">
              {view.slots.map((slot) => {
                const slotInterval = formatSlotInterval(
                  new Date(slot.startDateTime),
                  new Date(slot.endDateTime),
                );
                return (
                  <li key={slot.id}>
                    <button
                      type="button"
                      onClick={() =>
                        void handleReschedule(view.meeting, slot.id)
                      }
                      aria-label={`Перенести на ${slotInterval}`}
                      className="w-full rounded-xl border bg-card p-4 text-left text-card-foreground"
                    >
                      {slotInterval}
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
          <button
            type="button"
            onClick={() => setView({ kind: "meeting", meeting })}
            className="self-start rounded-lg border bg-background px-4 py-2"
          >
            Назад
          </button>
        </div>
      )}
    </div>
  );
}
