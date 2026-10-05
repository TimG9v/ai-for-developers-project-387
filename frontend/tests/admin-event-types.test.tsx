import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const eventTypesList = vi.fn();
const eventTypesCreate = vi.fn();

vi.mock("@/src/client", () => ({
  eventTypesList: (...args: unknown[]) => eventTypesList(...args),
  eventTypesCreate: (...args: unknown[]) => eventTypesCreate(...args),
  slotsCreate: vi.fn(),
  slotsList: vi.fn(),
}));

import { AdminEventTypes } from "@/components/admin-event-types";

afterEach(cleanup);

function fillForm({ title = "Созвон", description = "Короткий звонок", duration = "15" } = {}) {
  fireEvent.change(screen.getByLabelText("Название"), {
    target: { value: title },
  });
  fireEvent.change(screen.getByLabelText("Описание"), {
    target: { value: description },
  });
  fireEvent.change(screen.getByLabelText("Длительность, минут"), {
    target: { value: duration },
  });
}

function submit() {
  fireEvent.click(screen.getByRole("button", { name: "Создать тип встречи" }));
}

describe("admin page creates event types", () => {
  it("creates a type via the SDK and shows it in the list", async () => {
    const created = {
      id: "et9",
      title: "Созвон",
      description: "Короткий звонок",
      durationMinutes: 15,
    };
    eventTypesCreate.mockResolvedValueOnce({ data: created });
    eventTypesList.mockResolvedValueOnce({ data: [created] });

    render(<AdminEventTypes initialEventTypes={[]} />);
    fillForm();
    submit();

    await waitFor(() => {
      expect(eventTypesCreate).toHaveBeenCalledTimes(1);
    });
    expect(eventTypesCreate).toHaveBeenCalledWith({
      body: {
        id: expect.any(String),
        title: "Созвон",
        description: "Короткий звонок",
        durationMinutes: 15,
      },
    });
    await waitFor(() => {
      expect(screen.getByText("Созвон")).toBeTruthy();
    });
    expect(screen.getByText("15 мин.")).toBeTruthy();
  });

  it("trims title and description before sending", async () => {
    const created = {
      id: "et10",
      title: "Созвон",
      description: "Короткий звонок",
      durationMinutes: 15,
    };
    eventTypesCreate.mockResolvedValueOnce({ data: created });
    eventTypesList.mockResolvedValueOnce({ data: [created] });

    render(<AdminEventTypes initialEventTypes={[]} />);
    fillForm({ title: "  Созвон  ", description: " Короткий звонок " });
    submit();

    await waitFor(() => {
      expect(eventTypesCreate).toHaveBeenCalledTimes(1);
    });
    expect(eventTypesCreate).toHaveBeenCalledWith({
      body: {
        id: expect.any(String),
        title: "Созвон",
        description: "Короткий звонок",
        durationMinutes: 15,
      },
    });
  });

  it("blocks submit when title is empty", async () => {
    render(<AdminEventTypes initialEventTypes={[]} />);
    fillForm({ title: "" });
    submit();

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toBe(
      "Укажите название и длительность встречи",
    );
    expect(eventTypesCreate).not.toHaveBeenCalled();
  });

  it("blocks submit when duration is not positive", async () => {
    render(<AdminEventTypes initialEventTypes={[]} />);
    fillForm({ duration: "0" });
    submit();

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(eventTypesCreate).not.toHaveBeenCalled();
  });

  it("shows a server rejection message on 400", async () => {
    eventTypesCreate.mockResolvedValueOnce({ data: undefined, error: { status: 400 } });

    render(<AdminEventTypes initialEventTypes={[]} />);
    fillForm();
    submit();

    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(screen.getByRole("alert").textContent).toContain(
      "Не удалось создать тип встречи",
    );
  });

  it("renders the initial event types from the server", () => {
    render(
      <AdminEventTypes
        initialEventTypes={[
          { id: "et1", title: "Знакомство", durationMinutes: 30 },
        ]}
      />,
    );

    expect(screen.getByText("Знакомство")).toBeTruthy();
    expect(screen.getByText("30 мин.")).toBeTruthy();
  });
});
