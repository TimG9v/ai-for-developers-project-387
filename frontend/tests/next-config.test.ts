import { describe, expect, it } from "vitest";
import nextConfig from "@/next.config";

describe("next.config rewrites", () => {
  it("proxies /api/* and /health to in-container backend", async () => {
    const rewrites = await nextConfig.rewrites?.();
    expect(rewrites).toEqual([
      { source: "/api/:path*", destination: "http://127.0.0.1:8081/:path*" },
      { source: "/health", destination: "http://127.0.0.1:8081/health" },
    ]);
  });
});
