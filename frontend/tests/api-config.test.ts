import { describe, expect, it } from "vitest";
import { client } from "@/src/client/client.gen";
import "@/src/api-config";

describe("api-config", () => {
  it("browser (jsdom) gets same-origin /api as baseUrl", () => {
    // vitest: environment jsdom → window определён → браузерная ветка.
    expect(client.getConfig().baseUrl).toBe("/api");
  });
});
