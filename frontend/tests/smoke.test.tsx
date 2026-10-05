import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import Page from "@/app/page";

afterEach(cleanup);

describe("home page", () => {
  it("shows the service title and pitch", () => {
    render(<Page />);

    expect(
      screen.getByRole("heading", { level: 1, name: "Calendar" }),
    ).toBeTruthy();
    expect(
      screen.getByText(
        "Выберите свободный слот и запишитесь на 30-минутный звонок — без регистрации.",
      ),
    ).toBeTruthy();
  });

  it("links to the booking page", () => {
    render(<Page />);

    const link = screen.getByRole("link", { name: "Записаться" });
    expect(link.getAttribute("href")).toBe("/booking");
  });

  it("lists the service features", () => {
    render(<Page />);

    expect(
      screen.getByRole("heading", { name: "Возможности" }),
    ).toBeTruthy();
    expect(
      screen.getByText("Выбор свободного слота на странице записи."),
    ).toBeTruthy();
    expect(
      screen.getByText("Запись на 30-минутный звонок без регистрации."),
    ).toBeTruthy();
    expect(
      screen.getByText("Список предстоящих встреч у владельца календаря."),
    ).toBeTruthy();
  });
});
