import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const upcomingMeetingsList = vi.fn();
const bookingsCancel = vi.fn();
const routerRefresh = vi.fn();

vi.mock("@/src/client", () => ({
  upcomingMeetingsList: (...args: unknown[]) => upcomingMeetingsList(...args),
  eventTypesList: vi.fn(async () => ({ data: [] })),
  workingHoursApiGet: vi.fn(async () => ({
    data: { timeZone: "UTC", rules: [] },
  })),
  bookingsCancel: (...args: unknown[]) => bookingsCancel(...args),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: routerRefresh }),
}));

import Page from "@/app/admin/page";

const slotStart = new Date(2026, 10, 15, 10, 0);
const slotEnd = new Date(2026, 10, 15, 10, 30);

const MEETING = {
  id: "b1",
  slotId: "s1",
  guestName: "Гость",
  guestEmail: "g@example.com",
  startDateTime: slotStart.toISOString(),
  endDateTime: slotEnd.toISOString(),
  eventTypeId: "et1",
  eventTitle: "Созвон",
};

afterEach(cleanup);

describe("admin page lists upcoming meetings", () => {
  it("renders the meeting with guest, interval and event title", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });

    render(await Page());

    expect(screen.getByText("Гость")).toBeTruthy();
    expect(screen.getByText("g@example.com")).toBeTruthy();
    expect(screen.getByText(/15 ноября, 10:00–10:30/)).toBeTruthy();
    expect(screen.getByText(/Созвон · /)).toBeTruthy();
  });

  it("shows an empty state when there are no bookings", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [] });

    render(await Page());

    expect(screen.getByText("Пока нет записей")).toBeTruthy();
  });

  it("cancels only after the second confirming click", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    bookingsCancel.mockResolvedValueOnce({ data: undefined });

    render(await Page());
    fireEvent.click(screen.getByRole("button", { name: "Отменить запись b1" }));

    // Первый клик только спрашивает подтверждение — запись ещё на месте.
    const confirm = screen.getByRole("button", {
      name: "Подтвердите отмену записи b1",
    });
    expect(confirm.textContent).toContain("Точно отменить?");
    expect(bookingsCancel).not.toHaveBeenCalled();

    fireEvent.click(confirm);

    await waitFor(() => {
      expect(bookingsCancel).toHaveBeenCalledWith({ path: { id: "b1" } });
    });
    await waitFor(() => {
      expect(routerRefresh).toHaveBeenCalled();
    });
  });

  it("reverts the button when confirmation is not given in time", async () => {
    vi.useFakeTimers();
    try {
      upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });

      render(await Page());
      fireEvent.click(
        screen.getByRole("button", { name: "Отменить запись b1" }),
      );
      expect(
        screen.getByRole("button", { name: "Подтвердите отмену записи b1" }),
      ).toBeTruthy();

      act(() => {
        vi.advanceTimersByTime(4000);
      });

      expect(
        screen.getByRole("button", { name: "Отменить запись b1" }),
      ).toBeTruthy();
      expect(bookingsCancel).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it("explains a failed cancellation instead of breaking", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    bookingsCancel.mockResolvedValueOnce({ data: undefined, error: { status: 500 } });

    render(await Page());
    fireEvent.click(screen.getByRole("button", { name: "Отменить запись b1" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Подтвердите отмену записи b1" }),
    );

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain("Не удалось отменить");
    });
    expect(routerRefresh).not.toHaveBeenCalled();
  });
});
