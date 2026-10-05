# AGENTS.md

Календарь: бекенд Rust + axum (`backend/`), фронтенд Next.js 16 (App Router, TypeScript, Tailwind, shadcn/ui) (`frontend/`), CI — `.github/workflows/`.

## Команды

Запуск, тесты и линтеры — через Makefile в корне:

|       | backend                                                                                           | frontend                                       |
| ----- | ------------------------------------------------------------------------------------------------  | --------------------------------------------   |
| dev   | `make dev-backend` — `cargo run`, слушает `127.0.0.1:8081`                                        | `make dev-frontend` — `npm run dev`            |
| test  | `make test-backend` — `cargo test`                                                                | `make test-frontend` — `npm test` (vitest)     |
| lint  | `make lint-backend` — `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings`           | `make lint-frontend` — `npm run lint` (ESLint) |

Генерация из контракта — `make generate`: TypeSpec (`contracts/`) → OpenAPI
(`openapi/`) → клиентский SDK (`frontend/src/client/`) и серверные типы
(backend, OUT_DIR). Сгенерированные артефакты руками не править — только
перегенерация; источник правды — контракт в `contracts/`.

Оба приложения разом: `make dev`, `make test`, `make lint`; остановка dev-процессов — `make stop`.

## Структура

- `backend/` — crate `backend`: `src/lib.rs` (роутер `app()`), `src/main.rs` (bind), `tests/smoke.rs` (старт сервера, 200 на `GET /health`)
- `frontend/` — `app/` (страницы), `components/` (shadcn/ui), `tests/smoke.test.tsx`
- `.github/workflows/` — `backend-ci.yml` и `frontend-ci.yml` (тесты + линтеры на push и PR), `security.yml` (cargo audit / npm audit по lock-файлам), `release-please.yml` (release-PR после мержа в `main`)
- Пины версий: `.tool-versions` (rust), `.nvmrc` (node). Rust — в трёх местах: `.tool-versions` + `backend-ci.yml` + `security.yml`, менять все; node — только `.nvmrc` (CI читает файл напрямую).

## Правила

- Коммиты — Conventional Commits: `feat:`, `fix:`, `chore:`, `ci:`, `docs:` (scope допустим: `feat(backend):`). Формат касается и коммитов агента: release-please строит из истории коммитов changelog и semver-версию.
- Frontend — Next.js 16, отличается от привычной версии: перед правкой фронтенда читай гайд в `frontend/node_modules/next/dist/docs/` (блок правил автогенерируется `next dev` в `frontend/AGENTS.md`).

## Agent skills

### Issue tracker

Задачи живут в GitHub Issues этого репозитория (через `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Пять канонических ролей разбора, строки меток 1:1. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` + `docs/adr/` в корне репозитория. See `docs/agents/domain.md`.
