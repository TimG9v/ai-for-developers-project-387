import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const slotsList = vi.fn();

vi.mock("@/src/client", () => ({
  slotsList: (...args: unknown[]) => slotsList(...args),
}));

import { BookingEventTypes } from "@/components/booking-event-types";
import { formatSlotInterval } from "@/lib/slot-time";

const EVENT_TYPES = [
  { id: "et1", title: "Созвон", durationMinutes: 30 },
  { id: "et2", title: "Консультация", durationMinutes: 60 },
];

afterEach(cleanup);

// Локальные компоненты — рендер и ожидание в одной зоне, тест стабилен на CI.
const slotStart = new Date(2026, 10, 15, 10, 0);
const slotEnd = new Date(2026, 10, 15, 10, 30);

describe("booking page shows slot calendar per event type", () => {
  it("loads slots of the chosen type through the SDK and renders the interval", async () => {
    slotsList.mockResolvedValueOnce({
      data: [
        {
          id: "s1",
          eventTypeId: "et1",
          startDateTime: slotStart.toISOString(),
          endDateTime: slotEnd.toISOString(),
        },
      ],
    });

    render(<BookingEventTypes initialEventTypes={EVENT_TYPES} />);
    fireEvent.click(screen.getByRole("button", { name: "Выбрать Созвон" }));

    await waitFor(() => {
      expect(slotsList).toHaveBeenCalledWith({ query: { eventTypeId: "et1" } });
    });
    await waitFor(() => {
      expect(screen.getByText(formatSlotInterval(slotStart, slotEnd))).toBeTruthy();
    });
  });

  it("shows an empty state when the chosen type has no slots", async () => {
    slotsList.mockResolvedValueOnce({ data: [] });

    render(<BookingEventTypes initialEventTypes={EVENT_TYPES} />);
    fireEvent.click(screen.getByRole("button", { name: "Выбрать Консультация" }));

    await waitFor(() => {
      expect(slotsList).toHaveBeenCalledWith({ query: { eventTypeId: "et2" } });
    });
    await waitFor(() => {
      expect(screen.getByText("У этого типа пока нет свободных слотов")).toBeTruthy();
    });
  });

  it("renders the event types to choose from", () => {
    render(<BookingEventTypes initialEventTypes={EVENT_TYPES} />);

    expect(screen.getByRole("button", { name: "Выбрать Созвон" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Выбрать Консультация" })).toBeTruthy();
  });
});

describe("slot interval formatting", () => {
  it("formats start day and start–end time", () => {
    expect(formatSlotInterval(slotStart, slotEnd)).toBe("15 ноября, 10:00–10:30");
  });
});
