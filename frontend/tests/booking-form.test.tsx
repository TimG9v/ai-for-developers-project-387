import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const slotsList = vi.fn();
const bookingsCreate = vi.fn();

vi.mock("@/src/client", () => ({
  slotsList: (...args: unknown[]) => slotsList(...args),
  bookingsCreate: (...args: unknown[]) => bookingsCreate(...args),
}));

import { BookingEventTypes } from "@/components/booking-event-types";
import { formatSlotInterval } from "@/lib/slot-time";

const EVENT_TYPES = [{ id: "et1", title: "Созвон", durationMinutes: 30 }];

// Локальные компоненты — рендер и ожидание в одной зоне, тест стабилен на CI.
const slotStart = new Date(2026, 10, 15, 10, 0);
const slotEnd = new Date(2026, 10, 15, 10, 30);
const SLOT = {
  id: "s1",
  eventTypeId: "et1",
  startDateTime: slotStart.toISOString(),
  endDateTime: slotEnd.toISOString(),
};

afterEach(cleanup);

async function renderWithFreeSlot() {
  slotsList.mockResolvedValueOnce({ data: [SLOT] });
  render(<BookingEventTypes initialEventTypes={EVENT_TYPES} />);
  fireEvent.click(screen.getByRole("button", { name: "Выбрать Созвон" }));
  const slotButton = await screen.findByRole("button", {
    name: `Записаться на ${formatSlotInterval(slotStart, slotEnd)}`,
  });
  fireEvent.click(slotButton);
  await screen.findByLabelText("Имя");
}

describe("booking form", () => {
  it("opens the guest form when a slot is chosen", async () => {
    await renderWithFreeSlot();

    expect(screen.getByLabelText("Имя")).toBeTruthy();
    expect(screen.getByLabelText("Email")).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Записаться" }),
    ).toBeTruthy();
  });

  it("creates the booking via the SDK, shows confirmation and refreshes slots", async () => {
    bookingsCreate.mockResolvedValueOnce({
      data: {
        id: "b1",
        slotId: "s1",
        guestName: "Гость",
        guestEmail: "g@example.com",
        createdAt: "2026-11-01T09:00:00Z",
      },
    });

    await renderWithFreeSlot();
    // Повторная загрузка календаря после записи: слот занят и исчез.
    slotsList.mockResolvedValueOnce({ data: [] });
    fireEvent.change(screen.getByLabelText("Имя"), {
      target: { value: "Гость" },
    });
    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "g@example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Записаться" }));

    await waitFor(() => {
      expect(bookingsCreate).toHaveBeenCalledTimes(1);
    });
    const [{ body }] = bookingsCreate.mock.calls[0] as [
      { body: { id: string; slotId: string; guestName: string; guestEmail: string; createdAt: string } },
    ];
    expect(body.slotId).toBe("s1");
    expect(body.guestName).toBe("Гость");
    expect(body.guestEmail).toBe("g@example.com");
    expect(body.createdAt).toBeTruthy();

    await waitFor(() => {
      expect(screen.getByText(/Вы записаны/)).toBeTruthy();
    });
    // Занятый слот больше не выбирается в календаре.
    await waitFor(() => {
      expect(
        screen.queryByRole("button", { name: `Записаться на ${formatSlotInterval(slotStart, slotEnd)}` }),
      ).toBeNull();
    });
    await waitFor(() => {
      expect(slotsList).toHaveBeenCalledTimes(2);
    });
  });

  it("blocks submit when name or email is empty", async () => {
    await renderWithFreeSlot();
    fireEvent.change(screen.getByLabelText("Имя"), { target: { value: "" } });
    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Записаться" }));

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toBe("Укажите имя и email");
    expect(bookingsCreate).not.toHaveBeenCalled();
  });

  it("explains a 409 as slot already taken and refreshes the calendar", async () => {
    bookingsCreate.mockResolvedValueOnce({ data: undefined, error: { status: 409 } });

    await renderWithFreeSlot();
    // После 409 календарь перезагружается: занятый слот исчезает.
    slotsList.mockResolvedValueOnce({ data: [] });
    fireEvent.change(screen.getByLabelText("Имя"), { target: { value: "Гость" } });
    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "g@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Записаться" }));

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain("уже занято");
    });
    await waitFor(() => {
      expect(slotsList).toHaveBeenCalledTimes(2);
    });
    expect(
      screen.queryByRole("button", {
        name: `Записаться на ${formatSlotInterval(slotStart, slotEnd)}`,
      }),
    ).toBeNull();
  });

  it("explains a 404 as an outdated offer", async () => {
    bookingsCreate.mockResolvedValueOnce({ data: undefined, error: { status: 404 } });

    await renderWithFreeSlot();
    // Отказ сервера тоже перезагружает календарь: слот мог исчезнуть.
    slotsList.mockResolvedValueOnce({ data: [] });
    fireEvent.change(screen.getByLabelText("Имя"), { target: { value: "Гость" } });
    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "g@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Записаться" }));

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain("больше не доступен");
    });
  });
});
