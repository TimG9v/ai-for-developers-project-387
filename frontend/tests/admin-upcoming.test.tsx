import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const upcomingMeetingsList = vi.fn();

vi.mock("@/src/client", () => ({
  upcomingMeetingsList: (...args: unknown[]) => upcomingMeetingsList(...args),
  eventTypesList: vi.fn(async () => ({ data: [] })),
}));

import Page from "@/app/admin/page";

const slotStart = new Date(2026, 10, 15, 10, 0);
const slotEnd = new Date(2026, 10, 15, 10, 30);
const pastStart = new Date(2025, 0, 1, 10, 0);
const pastEnd = new Date(2025, 0, 1, 10, 30);

const EVENT_TYPES = [{ id: "et1", title: "Созвон", durationMinutes: 30 }];
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
});
