import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const workingHoursApiReplace = vi.fn();

vi.mock("@/src/client", () => ({
  workingHoursApiReplace: (...args: unknown[]) =>
    workingHoursApiReplace(...args),
}));

// Зона «браузера» владельца фиксирована: не путать с зоной сохранённого
// расписания в тестах ниже (Europe/Moscow).
vi.mock("@/lib/slot-time", () => ({
  viewerTimeZone: () => "Europe/Kaliningrad",
}));

import { AdminWorkingHours } from "@/components/admin-working-hours";

const SAVED = { timeZone: "Europe/Moscow", rules: [] };

afterEach(cleanup);

describe("admin working hours form", () => {
  it("defaults the zone to the browser zone while the schedule is unsaved", () => {
    render(
      <AdminWorkingHours initialWorkingHours={{ timeZone: "UTC", rules: [] }} />,
    );

    expect(
      (screen.getByLabelText("Часовой пояс") as HTMLSelectElement).value,
    ).toBe("Europe/Kaliningrad");
  });

  it("keeps the saved zone even when the windows are off", () => {
    render(<AdminWorkingHours initialWorkingHours={SAVED} />);

    expect(
      (screen.getByLabelText("Часовой пояс") as HTMLSelectElement).value,
    ).toBe("Europe/Moscow");
  });

  it("pre-fills from the saved schedule", () => {
    render(
      <AdminWorkingHours
        initialWorkingHours={{
          timeZone: "Europe/Moscow",
          rules: [
            { weekdays: [1, 2], startTime: "09:00", endTime: "12:00" },
          ],
        }}
      />,
    );

    expect(
      (screen.getByLabelText("Часовой пояс") as HTMLSelectElement).value,
    ).toBe("Europe/Moscow");
    expect((screen.getByLabelText("Пн") as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText("Ср") as HTMLInputElement).checked).toBe(false);
    expect((screen.getByLabelText("Начало окна") as HTMLInputElement).value).toBe(
      "09:00",
    );
  });

  it("saves weekdays, interval and time zone as a single rule", async () => {
    workingHoursApiReplace.mockResolvedValueOnce({ data: SAVED });

    render(<AdminWorkingHours initialWorkingHours={SAVED} />);
    for (const day of ["Пн", "Вт", "Ср", "Чт", "Пт"]) {
      fireEvent.click(screen.getByLabelText(day));
    }
    fireEvent.change(screen.getByLabelText("Начало окна"), {
      target: { value: "10:00" },
    });
    fireEvent.change(screen.getByLabelText("Конец окна"), {
      target: { value: "17:00" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Сохранить рабочие часы" }),
    );

    await waitFor(() => {
      expect(workingHoursApiReplace).toHaveBeenCalledTimes(1);
    });
    const [{ body }] = workingHoursApiReplace.mock.calls[0] as [
      { body: { timeZone: string; rules: { weekdays: number[]; startTime: string; endTime: string }[] } },
    ];
    expect(body.timeZone).toBe("Europe/Moscow");
    expect(body.rules).toEqual([
      { weekdays: [1, 2, 3, 4, 5], startTime: "10:00", endTime: "17:00" },
    ]);
    await waitFor(() => {
      expect(screen.getByText(/Расписание сохранено/)).toBeTruthy();
    });
  });

  it("disables windows with an empty rules list when no weekday is chosen", async () => {
    workingHoursApiReplace.mockResolvedValueOnce({ data: SAVED });

    render(<AdminWorkingHours initialWorkingHours={SAVED} />);
    fireEvent.click(screen.getByRole("button", { name: "Выключить рабочие окна" }));

    await waitFor(() => {
      expect(workingHoursApiReplace).toHaveBeenCalledTimes(1);
    });
    const [{ body }] = workingHoursApiReplace.mock.calls[0] as [
      { body: { timeZone: string; rules: unknown[] } },
    ];
    expect(body.rules).toEqual([]);
  });

  it("shows a server rejection message on 400", async () => {
    workingHoursApiReplace.mockResolvedValueOnce({
      data: undefined,
      error: {},
      response: { status: 400 },
    });

    render(<AdminWorkingHours initialWorkingHours={SAVED} />);
    fireEvent.click(screen.getByLabelText("Пн"));
    fireEvent.click(
      screen.getByRole("button", { name: "Сохранить рабочие часы" }),
    );

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toContain(
      "Сервер отклонил расписание",
    );
  });
});
