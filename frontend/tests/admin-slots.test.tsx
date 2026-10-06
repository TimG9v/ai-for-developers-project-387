import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const slotsCreate = vi.fn();

vi.mock("@/src/client", () => ({
  slotsCreate: (...args: unknown[]) => slotsCreate(...args),
  slotsList: vi.fn(),
}));

import { AdminSlots } from "@/components/admin-slots";

const EVENT_TYPES = [
  { id: "et1", title: "Созвон", durationMinutes: 30 },
  { id: "et2", title: "Консультация", durationMinutes: 60 },
];

afterEach(cleanup);

function fillSlotForm({
  eventTypeId = "et1",
  date = "2026-11-15",
  time = "10:00",
} = {}) {
  fireEvent.change(screen.getByLabelText("Тип встречи"), {
    target: { value: eventTypeId },
  });
  fireEvent.change(screen.getByLabelText("Дата"), {
    target: { value: date },
  });
  fireEvent.change(screen.getByLabelText("Время"), {
    target: { value: time },
  });
}

describe("admin page publishes slots", () => {
  it("предлагает 30-минутную сетку в поле времени", () => {
    render(<AdminSlots eventTypes={EVENT_TYPES} />);

    expect(screen.getByLabelText("Время").getAttribute("step")).toBe("1800");
  });

  it("creates a slot with interval duration from the chosen type", async () => {
    slotsCreate.mockResolvedValueOnce({
      data: {
        id: "s1",
        eventTypeId: "et1",
        startDateTime: "2026-11-15T10:00:00Z",
        endDateTime: "2026-11-15T10:30:00Z",
      },
    });

    render(<AdminSlots eventTypes={EVENT_TYPES} />);
    fillSlotForm();
    fireEvent.click(screen.getByRole("button", { name: "Опубликовать слот" }));

    await waitFor(() => {
      expect(slotsCreate).toHaveBeenCalledTimes(1);
    });
    const [{ body }] = slotsCreate.mock.calls[0] as [
      { body: { id: string; eventTypeId: string; startDateTime: string; endDateTime: string } },
    ];
    expect(body.eventTypeId).toBe("et1");
    const durationMs =
      new Date(body.endDateTime).getTime() - new Date(body.startDateTime).getTime();
    expect(durationMs).toBe(30 * 60 * 1000);
    await waitFor(() => {
      expect(screen.getByText("Слот опубликован")).toBeTruthy();
    });
  });

  it("blocks submit when type, date or time is missing", async () => {
    render(<AdminSlots eventTypes={EVENT_TYPES} />);
    fillSlotForm({ date: "" });
    fireEvent.click(screen.getByRole("button", { name: "Опубликовать слот" }));

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toBe(
      "Выберите тип встречи, дату и время",
    );
    expect(slotsCreate).not.toHaveBeenCalled();
  });

  it("shows a server rejection message on 400/404", async () => {
    slotsCreate.mockResolvedValueOnce({ data: undefined, error: { status: 400 } });

    render(<AdminSlots eventTypes={EVENT_TYPES} />);
    fillSlotForm();
    fireEvent.click(screen.getByRole("button", { name: "Опубликовать слот" }));

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toContain(
      "Не удалось опубликовать слот",
    );
  });
});
