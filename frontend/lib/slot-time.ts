/**
 * Интервал слота для календаря записи: «15 ноября, 10:00–10:30».
 * Дата берётся из начала; время — начало и конец интервала.
 */
export function formatSlotInterval(start: Date, end: Date): string {
  const day = new Intl.DateTimeFormat("ru-RU", {
    day: "numeric",
    month: "long",
  });
  const time = new Intl.DateTimeFormat("ru-RU", {
    hour: "2-digit",
    minute: "2-digit",
  });
  return `${day.format(start)}, ${time.format(start)}–${time.format(end)}`;
}
