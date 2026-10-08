# step-3-frontend-nextjs

- Дата: 2026-09-29 07:02
- Ветка: main
- Шаг: план шага 3/6 — фронтенд Next.js + shadcn/ui

Исходный план: `20260929062503-implementation-plan.md`.

## Цель

Работающий Next.js (App Router, TypeScript, Tailwind) с компонентом
shadcn/ui, smoke-тестом и рабочим ESLint. Ориентир — официальный
getting started Next.js (create-next-app).

## Действия

1. В корне: `npx create-next-app@latest frontend`
   (TypeScript, ESLint, Tailwind, App Router, npm — по текущим
   подсказкам create-next-app)
2. В `frontend/`: `npx shadcn@latest init`, затем
   `npx shadcn@latest add button`
3. `frontend/app/page.tsx` — вывести Button (shadcn/ui) на страницу
4. Smoke-тест:

   - devDeps: vitest, jsdom, @testing-library/react, @vitejs/plugin-react
   - `frontend/vitest.config.ts`: окружение jsdom, plugin-react
   - `frontend/tests/smoke.test.tsx`: рендер корневой страницы,
     assert: кнопка видна
   - скрипт `package.json`: `"test": "vitest run"`

5. Линтер — встроенный ESLint: `npm run lint`

## Файлы

- `frontend/*` — create-next-app
- `frontend/app/page.tsx` — правка
- `frontend/vitest.config.ts` — новый
- `frontend/tests/smoke.test.tsx` — новый
- `frontend/package.json` — правка (скрипт test, devDeps)

## Проверка

| # | Команда (real-run)                   | Ожидаемый результат |
| - | ------------------------------------ | ------------------- |
| 1 | cd frontend && npm test              | smoke-тест зелёный  |
| 2 | cd frontend && npm run lint          | exit 0              |
| 3 | cd frontend && npm run build         | сборка успешна      |
| 4 | npm run dev + открыть localhost:3000 | страница с кнопкой  |

## Коммит (делает пользователь)

`feat(frontend): next.js app with shadcn/ui and smoke test`

## Источники

- code-reading: `step-1.md` — shadcn/ui рекомендован, есть shadcn
  MCP Server; smoke-тест один
- Версии Next.js и shadcn не фиксируем: актуальные на момент реализации
  (требование шага — идти по документации)
