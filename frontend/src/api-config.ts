import { client } from "@/src/client/client.gen";

// Серверные компоненты (SSR) ходят напрямую на абсолютный BACKEND_URL
// (дефолт — dev-адрес из Makefile). Браузерные вызовы SDK идут на тот же
// origin (/api) — их проксирует next.config rewrites на backend: один
// origin, без CORS (тикет #20). В контейнере оба адреса ведут на один
// backend: SSR → 127.0.0.1:8081, браузер → /api → тот же 127.0.0.1:8081.
const isBrowser = typeof window !== "undefined";

client.setConfig({
  baseUrl: isBrowser
    ? "/api"
    : (process.env.BACKEND_URL ?? "http://127.0.0.1:8081"),
});
