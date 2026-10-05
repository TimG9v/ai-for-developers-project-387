import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const eventTypesList = vi.fn();

vi.mock("@/src/client", () => ({
  eventTypesList: (...args: unknown[]) => eventTypesList(...args),
  slotsList: vi.fn(),
}));

import Page from "@/app/booking/page";

afterEach(cleanup);

describe("booking page renders event types", () => {
  it("renders title, description and duration from the SDK", async () => {
    eventTypesList.mockResolvedValue({
      data: [
        {
          id: "et1",
          title: "Знакомство",
          description: "Первичный созвон",
          durationMinutes: 30,
        },
        {
          id: "et2",
          title: "Консультация",
          durationMinutes: 60,
        },
      ],
    });

    render(await Page());

    expect(
      screen.getByRole("heading", { level: 1, name: "Страница записи" }),
    ).toBeTruthy();
    expect(eventTypesList).toHaveBeenCalled();
    expect(screen.getByText("Знакомство")).toBeTruthy();
    expect(screen.getByText("Первичный созвон")).toBeTruthy();
    expect(screen.getByText("30 мин.")).toBeTruthy();
    expect(screen.getByText("Консультация")).toBeTruthy();
    expect(screen.getByText("60 мин.")).toBeTruthy();
  });

  it("renders an empty state when there are no event types", async () => {
    eventTypesList.mockResolvedValue({ data: [] });

    render(await Page());

    expect(
      screen.getByText("Пока нет доступных типов встреч"),
    ).toBeTruthy();
  });
});
