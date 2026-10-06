import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const upcomingMeetingsList = vi.fn();
const slotsList = vi.fn();
const bookingsCancel = vi.fn();
const bookingsReschedule = vi.fn();

vi.mock("@/src/client", () => ({
  upcomingMeetingsList: (...args: unknown[]) => upcomingMeetingsList(...args),
  slotsList: (...args: unknown[]) => slotsList(...args),
  bookingsCancel: (...args: unknown[]) => bookingsCancel(...args),
  bookingsReschedule: (...args: unknown[]) => bookingsReschedule(...args),
}));

import Page from "@/app/booking/manage/page";
import { BookingManage } from "@/components/booking-manage";
import { formatSlotInterval } from "@/lib/slot-time";

const slotStart = new Date(2026, 10, 15, 10, 0);
const slotEnd = new Date(2026, 10, 15, 10, 30);
const newSlotStart = new Date(2026, 10, 16, 11, 0);
const newSlotEnd = new Date(2026, 10, 16, 11, 30);
const interval = formatSlotInterval(slotStart, slotEnd);
const newSlotInterval = formatSlotInterval(newSlotStart, newSlotEnd);

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

const SLOT = {
  id: "s3",
  eventTypeId: "et1",
  startDateTime: newSlotStart.toISOString(),
  endDateTime: newSlotEnd.toISOString(),
};

afterEach(cleanup);

describe("booking manage", () => {
  it("shows booking details and offers cancel and reschedule", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });

    render(<BookingManage bookingId="b1" />);

    expect(await screen.findByText("«Созвон»")).toBeTruthy();
    expect(screen.getByText(new RegExp(interval))).toBeTruthy();
    expect(screen.getByText(/Гость/)).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Отменить запись" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Перенести запись" }),
    ).toBeTruthy();
  });

  it("explains a missing booking instead of breaking", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [] });

    render(<BookingManage bookingId="nope" />);

    expect(await screen.findByText("Запись не найдена")).toBeTruthy();
  });

  it("shows an error screen with retry when loading fails", async () => {
    upcomingMeetingsList.mockRejectedValueOnce(new Error("network down"));
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });

    render(<BookingManage bookingId="b1" />);

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Не удалось загрузить запись");
    expect(screen.queryByText("Загружаем запись…")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Повторить" }));

    expect(await screen.findByText("«Созвон»")).toBeTruthy();
    expect(upcomingMeetingsList).toHaveBeenCalledTimes(2);
  });

  it("stays on the error screen when the retry fails too", async () => {
    upcomingMeetingsList.mockRejectedValueOnce(new Error("network down"));
    upcomingMeetingsList.mockRejectedValueOnce(new Error("still down"));

    render(<BookingManage bookingId="b1" />);

    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button", { name: "Повторить" }));

    expect(await screen.findByText("Не удалось загрузить запись")).toBeTruthy();
  });

  it("shows missing for an empty id without fetching", async () => {
    render(<BookingManage bookingId="" />);

    expect(await screen.findByText("Запись не найдена")).toBeTruthy();
    expect(upcomingMeetingsList).not.toHaveBeenCalled();
  });

  it("cancels the booking by id from the manage link", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    bookingsCancel.mockResolvedValueOnce({ data: undefined });

    render(<BookingManage bookingId="b1" />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Отменить запись" }),
    );

    await waitFor(() => {
      expect(bookingsCancel).toHaveBeenCalledWith({ path: { id: "b1" } });
    });
    await waitFor(() => {
      expect(screen.getByText("Запись отменена")).toBeTruthy();
    });
  });

  it("explains a 404 on cancel as a missing booking", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    bookingsCancel.mockResolvedValueOnce({ data: undefined, error: { status: 404 } });

    render(<BookingManage bookingId="b1" />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Отменить запись" }),
    );

    await waitFor(() => {
      expect(screen.getByText("Запись не найдена")).toBeTruthy();
    });
  });

  it("reschedules to a free slot of the same event type", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    upcomingMeetingsList.mockResolvedValueOnce({
      data: [{ ...MEETING, slotId: "s3", startDateTime: newSlotStart.toISOString(), endDateTime: newSlotEnd.toISOString() }],
    });
    slotsList.mockResolvedValue({ data: [SLOT] });
    bookingsReschedule.mockResolvedValueOnce({
      data: { ...MEETING, slotId: "s3" },
    });

    render(<BookingManage bookingId="b1" />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Перенести запись" }),
    );

    const slotButton = await screen.findByRole("button", {
      name: `Перенести на ${newSlotInterval}`,
    });
    fireEvent.click(slotButton);

    await waitFor(() => {
      expect(bookingsReschedule).toHaveBeenCalledWith({
        path: { id: "b1" },
        body: { newSlotId: "s3" },
      });
    });
    await waitFor(() => {
      expect(screen.getByText(new RegExp(newSlotInterval))).toBeTruthy();
    });
  });

  it("explains a 409 on reschedule and reloads free slots", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });
    slotsList.mockResolvedValueOnce({ data: [SLOT] });
    slotsList.mockResolvedValueOnce({ data: [] });
    bookingsReschedule.mockResolvedValueOnce({
      data: undefined,
      error: { status: 409 },
    });

    render(<BookingManage bookingId="b1" />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Перенести запись" }),
    );

    const slotButton = await screen.findByRole("button", {
      name: `Перенести на ${newSlotInterval}`,
    });
    fireEvent.click(slotButton);

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain("уже занято");
    });
    await waitFor(() => {
      // Целевой слот после 409 уходит из перезагруженного календаря.
      expect(slotsList).toHaveBeenCalledTimes(2);
      expect(
        screen.queryByRole("button", { name: `Перенести на ${newSlotInterval}` }),
      ).toBeNull();
    });
  });
});

describe("booking manage page", () => {
  it("renders the heading and passes the id from search params", async () => {
    upcomingMeetingsList.mockResolvedValueOnce({ data: [MEETING] });

    render(
      await Page({ searchParams: Promise.resolve({ id: "b1" }) }),
    );

    expect(
      screen.getByRole("heading", { level: 1, name: "Управление записью" }),
    ).toBeTruthy();
    expect(await screen.findByText("«Созвон»")).toBeTruthy();
  });

  it("does not break without an id", async () => {
    render(await Page({ searchParams: Promise.resolve({}) }));

    expect(await screen.findByText("Запись не найдена")).toBeTruthy();
  });
});
