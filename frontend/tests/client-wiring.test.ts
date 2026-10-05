import { describe, expect, it } from "vitest";
import { client } from "@/src/client/client.gen";

// Проводка SDK (шаг 2 task-6, тикет #20): модульный граф клиентских
// компонентов должен сам настраивать same-origin baseUrl (/api) —
// api-config.ts не импортируется ничем серверным, что попадает в
// браузерный бандл (layout.tsx — серверный). Этот файл НЕ импортирует
// api-config напрямую: только компоненты.
import "@/components/admin-event-types";
import "@/components/admin-slots";
import "@/components/booking-event-types";
import "@/components/booking-form";

describe("client SDK wiring", () => {
  it("client component graph configures same-origin /api baseUrl", () => {
    expect(client.getConfig().baseUrl).toBe("/api");
  });
});
