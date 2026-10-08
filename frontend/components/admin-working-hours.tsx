"use client";

import { useState, type FormEvent } from "react";

import { workingHoursApiReplace, type WorkingHours } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

import { viewerTimeZone } from "@/lib/slot-time";

const WEEKDAYS = [
  { value: 1, short: "Пн" },
  { value: 2, short: "Вт" },
  { value: 3, short: "Ср" },
  { value: 4, short: "Чт" },
  { value: 5, short: "Пт" },
  { value: 6, short: "Сб" },
  { value: 7, short: "Вс" },
];

/** Список IANA-зон браузера; фолбэк — короткий набор, если API нет. */
function timeZoneOptions(): string[] {
  try {
    const supported = Intl.supportedValuesOf("timeZone");
    if (Array.isArray(supported) && supported.length > 0) {
      return [...supported];
    }
  } catch {
    // Intl.supportedValuesOf недоступен — используем фолбэк ниже.
  }
  return ["UTC", "Europe/Moscow", "Europe/Kaliningrad", "Asia/Yekaterinburg"];
}

/**
 * Рабочие часы владельца: дни недели + интервал + IANA-зона (дефолт — зона
 * браузера, пока расписание не сохранено). Сохранение — PUT /working-hours:
 * сервер валидирует расписание и материализует слоты всех типов встреч на
 * 14 дней вперёд в зоне владельца. Ручная публикация отдельных слотов
 * (AdminSlots) остаётся рядом для разовых встреч вне окон.
 */
export function AdminWorkingHours({
  initialWorkingHours,
}: {
  initialWorkingHours: WorkingHours;
}) {
  const firstRule = initialWorkingHours.rules[0];
  // Дефолт зоны — зона браузера владельца: вычисляется на клиенте (Intl),
  // потому что сервер не знает зону устройства. Применяется, пока расписание
  // не сохранено (это ещё GET-дефолт сервера «UTC без правил»); сохранённая
  // зона не подменяется, даже если окна выключены.
  const [timeZone, setTimeZone] = useState(
    initialWorkingHours.rules.length === 0 &&
      initialWorkingHours.timeZone === "UTC"
      ? viewerTimeZone()
      : initialWorkingHours.timeZone,
  );
  const [weekdays, setWeekdays] = useState<number[]>(firstRule?.weekdays ?? []);
  const [startTime, setStartTime] = useState(firstRule?.startTime ?? "10:00");
  const [endTime, setEndTime] = useState(firstRule?.endTime ?? "17:00");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  function touch() {
    setSaved(false);
  }

  function toggleWeekday(value: number) {
    touch();
    setWeekdays((current) =>
      current.includes(value)
        ? current.filter((day) => day !== value)
        : [...current, value].sort((a, b) => a - b),
    );
  }

  async function handleSubmit(formEvent: FormEvent<HTMLFormElement>) {
    formEvent.preventDefault();
    // Дни не выбраны — окна выключаются (rules: []): сервер удаляет
    // незанятые слоты окон; занятые и ручные слоты не трогает.
    const rules =
      weekdays.length === 0 ? [] : [{ weekdays, startTime, endTime }];
    const { error, response } = await workingHoursApiReplace({
      body: { timeZone, rules },
    });
    if (error !== undefined) {
      setErrorMessage(
        response?.status === 400
          ? "Сервер отклонил расписание: проверьте зону и интервал (кратен 30 минутам, начало раньше конца)"
          : "Не удалось сохранить расписание",
      );
      return;
    }
    setErrorMessage(null);
    setSaved(true);
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="flex w-full max-w-2xl flex-col gap-4 rounded-xl border bg-card p-6 text-card-foreground"
    >
      <h2 className="text-xl font-semibold">Рабочие часы</h2>
      <p className="text-sm text-muted-foreground">
        Один раз задайте рабочие окна — сервер опубликует слоты всех типов
        встреч на 14 дней вперёд в вашем часовом поясе. Если ни один день не
        выбран, окна выключены.
      </p>
      <fieldset className="flex flex-col gap-2">
        <legend className="text-sm font-medium">Дни недели</legend>
        <div className="flex flex-wrap gap-2">
          {WEEKDAYS.map(({ value, short }) => (
            <label
              key={value}
              className="flex items-center gap-1 rounded-lg border px-3 py-1"
            >
              <input
                type="checkbox"
                checked={weekdays.includes(value)}
                onChange={() => toggleWeekday(value)}
                aria-label={short}
              />
              {short}
            </label>
          ))}
        </div>
      </fieldset>
      <div className="flex flex-wrap gap-4">
        <div className="flex flex-1 flex-col gap-2">
          <label htmlFor="working-hours-start" className="text-sm font-medium">
            Начало окна
          </label>
          <input
            id="working-hours-start"
            type="time"
            step={1800}
            value={startTime}
            onChange={(changeEvent) => {
              touch();
              setStartTime(changeEvent.target.value);
            }}
            className="rounded-lg border bg-background px-3 py-2"
          />
        </div>
        <div className="flex flex-1 flex-col gap-2">
          <label htmlFor="working-hours-end" className="text-sm font-medium">
            Конец окна
          </label>
          <input
            id="working-hours-end"
            type="time"
            step={1800}
            value={endTime}
            onChange={(changeEvent) => {
              touch();
              setEndTime(changeEvent.target.value);
            }}
            className="rounded-lg border bg-background px-3 py-2"
          />
        </div>
        <div className="flex flex-1 flex-col gap-2">
          <label htmlFor="working-hours-zone" className="text-sm font-medium">
            Часовой пояс
          </label>
          <select
            id="working-hours-zone"
            value={timeZone}
            onChange={(changeEvent) => {
              touch();
              setTimeZone(changeEvent.target.value);
            }}
            className="rounded-lg border bg-background px-3 py-2"
          >
            {timeZoneOptions().map((zone) => (
              <option key={zone} value={zone}>
                {zone}
              </option>
            ))}
          </select>
        </div>
      </div>
      {errorMessage && (
        <p role="alert" className="text-sm text-destructive">
          {errorMessage}
        </p>
      )}
      {saved && (
        <p className="text-sm text-muted-foreground">
          Расписание сохранено — слоты опубликованы на 14 дней
        </p>
      )}
      <button
        type="submit"
        className="rounded-lg bg-primary px-4 py-2 text-primary-foreground"
      >
        {weekdays.length === 0
          ? "Выключить рабочие окна"
          : "Сохранить рабочие часы"}
      </button>
    </form>
  );
}
