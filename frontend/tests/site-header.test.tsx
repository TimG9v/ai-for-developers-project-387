import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { SiteHeader } from "@/components/site-header";

afterEach(cleanup);

describe("site header", () => {
  it("shows the service logo linking home", () => {
    render(<SiteHeader />);

    const logo = screen.getByRole("link", { name: "Calendar" });
    expect(logo.getAttribute("href")).toBe("/");
  });

  it("links to the booking page", () => {
    render(<SiteHeader />);

    const link = screen.getByRole("link", { name: "Записаться" });
    expect(link.getAttribute("href")).toBe("/booking");
  });

  it("links to the admin page", () => {
    render(<SiteHeader />);

    const link = screen.getByRole("link", { name: "Админка" });
    expect(link.getAttribute("href")).toBe("/admin");
  });
});
