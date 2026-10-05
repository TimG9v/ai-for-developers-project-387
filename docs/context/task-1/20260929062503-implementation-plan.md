# implementation-plan

- Дата: 2026-09-29 06:25
- Ветка: master
- Шаг: план реализации

## Задача

Шаг 1 «Каркас приложения» (`context/tmp/task-1/step-1.md`): собрать каркас
приложения без функциональности — бекенд, фронтенд, тестовый раннер, линтеры,
CI в GitHub Actions, release-please, AGENTS.md. Главный ориентир — официальный
getting started каждого инструмента, а не память модели о версиях.

## Стек (решение принято пользователем)

- Бекенд: **Rust + axum** (Tokio) — выбор пользователя
- Фронтенд: **Next.js** (App Router, TypeScript, Tailwind) — выбор пользователя
- UI: **shadcn/ui** — подтверждено пользователем

Отличие от рекомендации курса (TS + Vite + shadcn/Mantine) осознанное:
бекенд берётся на Rust. shadcn/ui ложится на Next.js штатно: официальный
init-флоу написан под Next.js, есть shadcn MCP Server для агента.

## Окружение (проверено командами 2026-09-29, real-run)

| Инструмент | Версия                              | Источник                    |
| ---------- | ----------------------------------- | --------------------------- |
| Node.js    | v25.2.0                             | real-run: node --version    |
| npm        | 11.6.2                              | real-run: npm --version     |
| Python     | 3.14.4                              | real-run: python3 --version |
| Rust       | 1.98.1 (asdf; также 1.98.0, 1.96.1) | real-run: asdf list rust    |
| Ruby, Go   | не установлены                      | real-run: which → нет       |

Состояние репозитория (real-run: git status / git remote -v / git branch --show-current):

- Репозиторий пуст: коммитов нет; в `.gitignore` только `context/tmp*`
- Git-remote не настроен — для CI и release-please нужен URL GitHub-репозитория
- Текущая ветка `master`; по плану переименовать в `main` до первого коммита
- Версию Rust в проекте зафиксировать через `.tool-versions` (asdf)

## Структура репозитория

```text
calendar/
├── .tool-versions               # rust 1.98.1
├── .gitignore                   # дополнить: target/, node_modules/, .next/
├── .github/workflows/
│   ├── ci.yml                   # тесты + линтеры на push и pull_request
│   └── release-please.yml       # release-please отдельным workflow
├── .release-please-manifest.json
├── release-please-config.json
├── Makefile                     # dev / test / lint для обоих приложений
├── AGENTS.md                    # команды, устройство, правило коммитов
├── backend/                     # Rust + axum
│   ├── src/main.rs              # GET /health → JSON
│   ├── tests/smoke.rs           # smoke: старт сервера, ответ 200
│   └── Cargo.toml
└── frontend/                    # Next.js (App Router, TS, Tailwind)
    ├── app/page.tsx             # страница с компонентом shadcn/ui (Button)
    ├── components/              # shadcn/ui
    ├── tests/smoke.test.tsx     # smoke: root-страница рендерится
    └── package.json
```

## Последовательность работ (каждый пункт — отдельный Conventional Commit)

1. `chore:` — инициализация: `.gitignore` (target/, node_modules/, .next/),
   `.tool-versions` (rust 1.98.1), ветка `master` → `main`
2. `feat(backend):` — axum по официальному getting started: `GET /health` → JSON;
   интеграционный smoke-тест (старт сервера, ответ 200); линтеры
   `cargo clippy -- -D warnings` и `cargo fmt --check`
3. `feat(frontend):` — `create-next-app` (TS, ESLint, Tailwind), `shadcn init`
   + компонент Button; smoke-тест на vitest (root-страница рендерится);
   линтер — встроенный ESLint
4. `ci:` — `.github/workflows/ci.yml`: trigger на push и pull_request;
   job rust (fmt --check, clippy -D warnings, cargo test) и
   job frontend (npm ci, lint, test); прогон должен быть зелёным
5. `chore:` — release-please отдельным workflow
   (`.github/workflows/release-please.yml`, release-please-action v4)
   + `.release-please-manifest.json` + config в корне
6. `docs:` — AGENTS.md: команды запуска/тестов/линтеров, устройство проекта,
   правило Conventional Commits (feat:, fix: и т. д.)

## Команды

- Backend: `cd backend && cargo run`, `cargo test`,
  `cargo clippy -- -D warnings`, `cargo fmt --check`
- Frontend: `cd frontend && npm run dev`, `npm test`, `npm run lint`
- Корень: `Makefile` с целями dev / test / lint для обоих приложений

## Открытые вопросы (блокируют части шага)

1. **GitHub-репозиторий**: remote не настроен; CI и release-please
   проверяются только на реальном push — нужен URL репозитория.
   Push выполняет пользователь (у агента git read-only).
2. **Коммиты**: по правилам проекта агент не коммитит; точные
   коммит-сообщения даны выше, коммиты делает пользователь.

## Критерии приёмки (из step-1.md, для финальной проверки)

- Бекенд и фронтенд запускаются локально
- В проекте есть команды запуска тестов и линтера
- GitHub Actions прогоняет тесты и линтер на каждый push, прогон зелёный
- Коммиты по Conventional Commits; release-please создаёт release-PR
  после мержа в основную ветку
- В корне есть AGENTS.md с командами и правилом про формат коммитов
- После каркаса запущен инициализация агента (/init), AGENTS.md
  сокращён до рабочего минимума

## Источники выводов

- real-run: версии интерпретаторов, состояние git-репозитория,
  список rust в asdf — команды, выполненные 2026-09-29
- code-reading: требования шага и порядок работ — `context/tmp/task-1/step-1.md`
- конкретные версии axum / Next.js / release-please-action намеренно не
  зафиксированы: выбираются по актуальному официальному getting started
  во время реализации — это прямое требование самого шага
