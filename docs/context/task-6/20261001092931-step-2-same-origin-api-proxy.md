# step-2-same-origin-api-proxy

- Дата: 2026-10-01 09:29
- Ветка: task-6 (рабочая)
- Шаг: план шага 2/5 — API на том же origin: Next rewrites + baseUrl SDK

Исходный план: `20261001092929-implementation-plan.md`.

## Цель

Браузерные вызовы SDK идут на свой origin (`/api/*`), Next.js проксирует их
на backend (`127.0.0.1:8081`); SSR-вызовы ходят напрямую на абсолютный
`BACKEND_URL`. Один origin убирает CORS — это закрывает корень тикета #20 и
создаёт предпосылку деплоя: публичный контейнер отдаёт страницы и API через
один порт. Локальный prod-прогон (build + start) подтверждает: `POST
/api/event-types` работает, `/health` отдаёт статус, страницы рендерятся.

## Предусловия (real-run перед стартом)

1. Baseline зелёный: `make test && make lint` — exit 0.
2. Шаг 1 выполнен (backend слушает `BACKEND_PORT`, дефолт 8081 — совместимо
   с текущим dev-сценарием без переменных окружения).
3. Тикет #20 открыт (`gh issue view 20` — состояние `OPEN`): шаг закрывает
   его корень; закрытие — в шаге 5 (после браузерного прогона).

## Действия (TDD)

1. **Failing-тест на браузерную ветку baseUrl** — новый файл
   `frontend/tests/api-config.test.ts`:

   ```ts
   import { describe, expect, it } from "vitest";
   import { client } from "@/src/client/client.gen";
   import "@/src/api-config";

   describe("api-config", () => {
     it("browser (jsdom) gets same-origin /api as baseUrl", () => {
       // vitest: environment jsdom → window определён → браузерная ветка.
       expect(client.getConfig().baseUrl).toBe("/api");
     });
   });
   ```

   Vitest изолирует реестр модулей на файл — импорт синглтона `client` в этом
   файле даёт чистую конфигурацию; на другие тест-файлы не влияет.
2. **Failing-тест на rewrites** — новый файл `frontend/tests/next-config.test.ts`:

   ```ts
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
   ```

3. **Проверить, что падают**:

   ```bash
   cd frontend && npx vitest run tests/api-config.test.ts tests/next-config.test.ts
   ```

   Ожидание: FAIL — baseUrl сейчас `http://127.0.0.1:8081`, `rewrites` нет.
4. **Реализация** — `frontend/src/api-config.ts`:

   ```ts
   import { client } from "@/src/client/client.gen";

   // Серверные компоненты (SSR) ходят напрямую на абсолютный BACKEND_URL
   // (дефолт — dev-адрес из Makefile). Браузерные вызовы SDK идут на тот же
   // origin (/api) — их проксирует next.config rewrites на backend: один
   // origin, без CORS (тикет #20). В контейнере оба адреса совпадают по
   // сути: SSR → 127.0.0.1:8081, браузер → /api → тот же 127.0.0.1:8081.
   const isBrowser = typeof window !== "undefined";

   client.setConfig({
     baseUrl: isBrowser
       ? "/api"
       : (process.env.BACKEND_URL ?? "http://127.0.0.1:8081"),
   });
   ```

   `frontend/next.config.ts`:

   ```ts
   import type { NextConfig } from "next";

   // Прокси API на внутренний backend. Цель — константа: Next разрешает
   // rewrites при `next build` (routes-manifest), runtime-переменные здесь
   // не работают. Топология контейнера фиксирована entrypoint'ом (шаг 3):
   // backend всегда на 127.0.0.1:8081.
   const backend = "http://127.0.0.1:8081";

   const nextConfig: NextConfig = {
     async rewrites() {
       return [
         { source: "/api/:path*", destination: `${backend}/:path*` },
         { source: "/health", destination: `${backend}/health` },
       ];
     },
   };

   export default nextConfig;
   ```

5. **Зелёный**:

   ```bash
   cd frontend && npx vitest run tests/api-config.test.ts tests/next-config.test.ts
   ```

   Ожидание: PASS (2 теста). Затем `npm test` — все 32 прежних + 2 новых.
6. **Локальный prod-прогон** (реальный шов прокси; dev-режим тоже получает
   rewrites, но проверяем боевой путь):

   ```bash
   make stop
   cd backend && cargo run &
   cd frontend && npm run build && npm run start &
   ```

   Проверки curl (real-run):

   ```bash
   curl -sf http://127.0.0.1:3000/health                          # {"status":"ok"}
   curl -sf -X POST http://127.0.0.1:3000/api/event-types \
     -H 'content-type: application/json' \
     -d '{"id":"et-proxy","title":"Проксирование","durationMinutes":30}'   # 2xx
   curl -sf http://127.0.0.1:3000/api/event-types                 # тип виден
   curl -sf http://127.0.0.1:3000/booking | grep -q "Проксирование"        # SSR видит
   ```

   Прямой вызов тоже должен работать (SSR-путь): `curl -sf
   http://127.0.0.1:3000/booking` уже это покрыл — отдельный curl на 8081 не
   нужен (порт не публикуется).
7. **Браузерная проверка** (по возможности в сессии): playwright-core во
   временном каталоге вне репо (практика task-5) либо встроенные браузерные
   инструменты — открыть `http://127.0.0.1:3000/admin`, создать тип из формы;
   Network: запрос уходит на `http://127.0.0.1:3000/api/event-types` (тот же
   origin) и получает 2xx. Это фактическое закрытие #20 в браузере.
8. Полный прогон: `make test && make lint` — exit 0.
9. Коммит:

   ```bash
   git add frontend/next.config.ts frontend/src/api-config.ts \
     frontend/tests/api-config.test.ts frontend/tests/next-config.test.ts
   git commit -m "fix(frontend): same-origin /api proxy via rewrites (Closes #20)"
   ```

   (Issue закроется автоматически при merge PR в main — шаг 5.)

## Файлы

- Modify: `frontend/next.config.ts` (rewrites), `frontend/src/api-config.ts`
  (развилка сервер/браузер).
- Create: `frontend/tests/api-config.test.ts`,
  `frontend/tests/next-config.test.ts`.
- Не трогать: `frontend/src/client/**` (генерат), `backend/**` (шаг 1 уже
  закрыт), `openapi/**`, `contracts/**`, Makefile (dev-сценарий не меняется).

## Проверка (real-run)

| # | Команда                                                        | Ожидаемый результат                                    |
| - | -------------------------------------------------------------- | ------------------------------------------------------ |
| 1 | `cd frontend && npx vitest run tests/api-config.test.ts`        | PASS: baseUrl `/api` в jsdom                            |
| 2 | `cd frontend && npx vitest run tests/next-config.test.ts`       | PASS: два правила rewrites                              |
| 3 | `npm test` (в frontend)                                        | PASS: 34 теста (32 + 2 новых)                           |
| 4 | prod-прогон: `curl /health` через :3000                        | `{"status":"ok"}`                                       |
| 5 | `POST /api/event-types` + повторный `GET /api/event-types`      | 2xx, созданный тип в списке                             |
| 6 | `curl /booking`                                                | 200, SSR-страница содержит созданный тип                |
| 7 | браузер: форма `/admin` создаёт тип                             | 2xx на `/api/event-types` (same origin, Network)        |
| 8 | `make test && make lint`                                       | exit 0                                                  |
| 9 | `git status --porcelain` после коммита                          | пусто                                                   |

## Коммит

Агент в ветке `task-6`: `fix(frontend): same-origin /api proxy via rewrites
(Closes #20)`. Если браузерная проверка (п. 7) в сессии недоступна — в коммит
не класть «Closes #20», закрытие перенести в шаг 5 после проверки на
публичном URL (в комментарии коммита тогда просто `... via rewrites`).

## Предусловия следующих шагов

Шаг 3 упаковывает в образ ровно это поведение: next на `$PORT` с rewrites на
`127.0.0.1:8081`, backend на `BACKEND_PORT` (дефолт 8081). Если шаг 2 не
прошёл браузерную проверку — шаг 4 (деплой) не начинать: публичный сайт
унаследует дефект.

## Источники

- real-run (чтение файлов, 2026-10-01): `frontend/src/api-config.ts` (текущий
  безусловный `BACKEND_URL ?? :8081`), `frontend/next.config.ts` (пустой),
  `frontend/src/client/client/utils.gen.ts` (`createConfig` без дефолтного
  baseUrl), `frontend/vitest.config.ts` (jsdom, alias `@`), `Makefile`
  (dev-адреса 3000/8081).
- real-run (чтение, 2026-10-01): `docs/context/task-5/
  20261001074828-step-6-e2e-report.md` — факты CORS (preflight 405, B1–B4/B7
  недостижимы из браузера), практика prod-старта для браузерных прогонов;
  тело issue #20 (`gh issue view 20`) — два варианта решения, выбран прокси.
- code-reading: изоляция модулей vitest на файл (синглтон `client` в тесте
  чистый); `typeof window` как развилка SSR/браузер — стандартный приём
  Next; запекание rewrites при сборке — учитывается выбором константы и
  проверяется п. 5 таблицы.
