# step-1-ticket-14-event-types

- Дата: 2026-09-30 17:39
- Ветка: task-5 (рабочая)
- Шаг: план шага 1/7 — /implement тикета #14 «Виды брони: список типов встреч»

Исходный план: `20260930173909-implementation-plan.md`.

## Цель

Первый тикет фронтира реализован: `GET /event-types` отдаёт список типов встреч,
страница записи (`/booking`) рендерит виды брони — название, описание, длительность —
полученные через сгенерированный SDK. Тесты обоих швов зелёные. Тикет #14 закрыт.

## Предусловия (real-run перед стартом)

1. Бaseline зелёный:

   ```bash
   make test && make lint
   ```

   Ожидание: exit 0 (на 2026-09-30 — 10 тестов фронта, cargo-тесты бекенда).
2. Тикет #14 открыт, блокирующих нет:

   ```bash
   gh api repos/TimG9v/ai-for-developers-project-386/issues/14/dependencies/blocked_by
   ```

   Ожидание: пустой массив.
3. Свежая сессия: прочитаны `CONTEXT.md` (словарь: Тип встречи, Слот, Запись),
   спека #13 (Testing Decisions — швы), тело тикета #14.

## Действия

1. Пользователь запускает `/implement` с тикетом: «реализуй #14 из спеки #13».
2. Агент по скиллу читает тикет, спеку #13 и `CONTEXT.md`; TDD на швах:

   - **Backend (HTTP-шов, axum):** failing-тест в `backend/tests/` — запрос
     `GET /event-types` к `app()` с тестовым состоянием; ожидание 200 и JSON
     `EventType[]` в форме контракта (`id`, `title`, `description?`,
     `durationMinutes`). Хендлер поверх in-memory репозитория за трейтом
     (решение карты: «in-memory + trait-репозиторий») — трейт в приложении,
     имплементация in-memory в инфраструктуре. Типы ответа — из
     `backend::api::api_types` (руками не править).
   - **Frontend (страница + мок SDK):** failing-тест в `frontend/tests/` —
     `/booking` рендерит список типов (название, описание, длительность в
     минутах, текст на русском — ADR 0001); SDK-модуль `../src/client`
     мокается через `vi.mock` (шов спеки: «страницы с моком SDK»), вызов
     `eventTypesList()`. Страница вызывает SDK, никакого рукописного `fetch`.
3. Красный → зелёный по циклу `/tdd`; typecheck и точечные тесты по ходу
   (`cargo test --test <file>`, `npx vitest run <file>`); состояние в роутере —
   `AppState` c репозиторием, `app()` собирает роутер с состоянием.
4. Полный прогон в конце: `make test`, `make lint` — exit 0.
5. `/code-review` внутри `/implement`; замечания разбираются в той же сессии.
6. Коммиты — `/implement` коммитит сам (Conventional Commits, ветка `task-5`).
7. Проверка результата (`descripton.md`: «от вас нужно проверить, что
   получилось, и вернуть замечания»): прогон из раздела «Проверка», сверка
   acceptance-критериев тикета. Замечания — в issue #14, исправления —
   в той же сессии.
8. Закрыть тикет (освобождает фронтир для #15):

   ```bash
   gh issue close 14 --comment "реализовано: <коммиты>, тесты обоих швов зелёные"
   ```

## Файлы (ожидаемая форма)

- Modify: `backend/src/lib.rs` (роутер + состояние), `backend/src/` — новые
  модули домена (трейт репозитория + in-memory).
- Modify: `frontend/app/booking/page.tsx` (заглушка → список видов брони).
- Create: `backend/tests/event_types.rs` (или расширение существующих),
  `frontend/tests/booking-event-types.test.tsx`.
- Не трогать: `frontend/src/client/**`, `backend/src/api.rs`, `openapi/**`
  (контракт на этом тикете не меняется — 400 не нужен, только list).

## Проверка (real-run)

| # | Команда                                                                                                                                                             | Ожидаемый результат                                                                                          |
| - | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| 1 | `make test`                                                                                                                                                         | exit 0; новые тесты обоих швов зелёные                                                                       |
| 2 | `make lint`                                                                                                                                                         | exit 0                                                                                                       |
| 3 | `make dev` + `curl -s http://127.0.0.1:8081/event-types`                                                                                                            | 200, `[]` (пустой список — валидный ответ)                                                                   |
| 4 | `curl -s -X POST http://127.0.0.1:8081/event-types -H 'content-type: application/json' -d '{"id":"et1","title":"Звонок","durationMinutes":30}'` затем повторный GET | тип виден (создание придёт в #15; здесь — ручная дырка допустима, если сессия решит; минимум — GET отвечает) |
| 5 | страница `/booking` в браузере                                                                                                                                      | заглушка заменена списком видов брони                                                                        |
| 6 | `git diff --stat` против сгенерированных путей                                                                                                                      | `frontend/src/client/**`, `openapi/**`, `backend/src/api.rs` не изменены                                     |
| 7 | `gh api repos/TimG9v/ai-for-developers-project-386/issues/14 --jq .state`                                                                                           | `closed`                                                                                                     |

## Коммит

Выполняет `/implement` в ветке `task-5`. Формат: `feat: ...` / `test: ...`
(Conventional Commits — AGENTS.md). Шаг ничего не коммитит поверх.

## Предусловия следующих шагов

Шаг 2 берёт #15: единственный блокер #14 закрыт. `make generate` не требуется,
если контракт не менялся.

## Источники

- real-run (чтение файлов, 2026-09-30): `.agents/skills/implement/SKILL.md`
  (TDD, швы, полный прогон, ревью, коммит; `disable-model-invocation: true`);
  тело issue #14 (acceptance criteria); спека #13 (Testing Decisions — швы).
- real-run (shell, 2026-09-30): `make test`/`make lint` exit 0 (baseline);
  `gh api …/issues/14/dependencies/blocked_by` — пусто.
- code-reading: состав тестов и файлов — из текущих заглушек
  (`backend/src/lib.rs` — только /health; `frontend/app/booking/page.tsx`) и
  Testing Decisions спеки.
