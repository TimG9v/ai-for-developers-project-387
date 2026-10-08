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

/**
 * Часовой пояс просматривающего (зона устройства). Фолбэк UTC — если
 * браузер не сообщил зону.
 */
export function viewerTimeZone(): string {
  try {
    const resolved = new Intl.DateTimeFormat().resolvedOptions().timeZone;
    return resolved === undefined || resolved === "" ? "UTC" : resolved;
  } catch {
    return "UTC";
  }
}

/**
 * Интервал слота в заданной зоне с явной подписью часового пояса:
 * «15 ноября, 10:00–10:30 (GMT+3, Europe/Moscow)». Гость из другого
 * региона видит и своё время, и чей это час — подпись исключает догадки.
 */
export function formatSlotIntervalInZone(
  start: Date,
  end: Date,
  timeZone: string,
): string {
  const day = new Intl.DateTimeFormat("ru-RU", {
    day: "numeric",
    month: "long",
    timeZone,
  });
  const time = new Intl.DateTimeFormat("ru-RU", {
    hour: "2-digit",
    minute: "2-digit",
    timeZone,
  });
  const zoneName =
    new Intl.DateTimeFormat("ru-RU", { timeZone, timeZoneName: "shortOffset" })
      .formatToParts(start)
      .find((part) => part.type === "timeZoneName")?.value ?? timeZone;
  return `${day.format(start)}, ${time.format(start)}–${time.format(end)} (${zoneName}, ${timeZone})`;
}
