import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/src/client", () => ({
  eventTypesList: vi.fn(async () => ({ data: [] })),
  slotsList: vi.fn(),
}));

import Page from "@/app/booking/page";

afterEach(cleanup);

describe("booking page", () => {
  it("renders the page heading", async () => {
    render(await Page());

    expect(
      screen.getByRole("heading", { level: 1, name: "Страница записи" }),
    ).toBeTruthy();
  });
});
