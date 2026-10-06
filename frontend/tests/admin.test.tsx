import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/src/client", () => ({
  eventTypesList: vi.fn(async () => ({ data: [] })),
  slotsList: vi.fn(async () => ({ data: [] })),
  upcomingMeetingsList: vi.fn(async () => ({ data: [] })),
  bookingsCancel: vi.fn(async () => ({ data: undefined })),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: vi.fn() }),
}));

import Page from "@/app/admin/page";

afterEach(cleanup);

describe("admin page", () => {
  it("renders the page heading", async () => {
    render(await Page());

    expect(
      screen.getByRole("heading", { level: 1, name: "Админка" }),
    ).toBeTruthy();
  });
});
