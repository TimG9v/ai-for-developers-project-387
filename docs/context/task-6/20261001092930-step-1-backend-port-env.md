# step-1-backend-port-env

- Дата: 2026-10-01 09:29
- Ветка: task-6 (рабочая)
- Шаг: план шага 1/5 — порт backend из переменной окружения

Исходный план: `20261001092929-implementation-plan.md`.

## Цель

Backend слушает порт из переменной окружения `BACKEND_PORT` (по умолчанию
8081), выбор адреса вынесен в чистую функцию с юнит-тестами. В контейнере
(шаг 3) entrypoint задаёт `BACKEND_PORT=8081` явно, `PORT` остаётся
исключительно портом Next.js — коллизия портов исключена по построению.

## Предусловия (real-run перед стартом)

1. Baseline зелёный:

   ```bash
   make test && make lint
   ```

   Ожидание: exit 0 (на 2026-10-01: 24 backend-теста, 32 frontend-теста;
   3 предсуществующих eslint-предупреждения не блокируют).
2. Ветка `task-6`, дерево чистое: `git status --porcelain` — пусто.
3. Свежая сессия: прочитаны `CONTEXT.md`, этот файл и общий план.

## Действия (TDD)

1. **Failing-тест** — юнит-тесты в `backend/src/lib.rs` (сейчас в lib-секции
   `cargo test` 0 тестов — место под них свободное):

   ```rust
   #[cfg(test)]
   mod bind_addr_tests {
       use super::bind_addr;

       #[test]
       fn default_is_loopback_8081() {
           assert_eq!(bind_addr(None), "127.0.0.1:8081");
       }

       #[test]
       fn backend_port_overrides_default() {
           assert_eq!(bind_addr(Some("9000")), "127.0.0.1:9000");
       }
   }
   ```

   Функция чистая (значение передаётся параметром, env не читается внутри) —
   тесты не гоняют env и не зависят от порядка.
2. **Проверить, что падает**:

   ```bash
   cd backend && cargo test --lib
   ```

   Ожидание: FAIL — `bind_addr` не существует (compile error — тоже «красный»).
3. **Минимальная реализация** в `backend/src/lib.rs` (вне тестового модуля):

   ```rust
   /// Адрес для TcpListener::bind: loopback, порт из BACKEND_PORT
   /// (дефолт 8081). PORT — не здесь: это публичный порт Next.js в контейнере.
   pub fn bind_addr(backend_port: Option<&str>) -> String {
       format!("127.0.0.1:{}", backend_port.unwrap_or("8081"))
   }
   ```

   И в `backend/src/main.rs` — чтение env на границе:

   ```rust
   let backend_port = std::env::var("BACKEND_PORT").ok();
   let bind_addr = backend::bind_addr(backend_port.as_deref());
   let listener = tokio::net::TcpListener::bind(&bind_addr)
       .await
       .unwrap_or_else(|err| panic!("bind {bind_addr}: {err}"));
   ```

   Невалидное значение `BACKEND_PORT` (например `abc`) не валидируем заранее —
   `bind` упадёт с понятным сообщением в панике; ранняя валидация — YAGNI.
4. **Зелёный**:

   ```bash
   cd backend && cargo test --lib
   ```

   Ожидание: PASS (2 теста).
5. Полный прогон: `make test && make lint` — exit 0.
6. Ручная smoke-проверка оверрайда (real-run, опционально, но дёшево):

   ```bash
   cd backend && BACKEND_PORT=9091 cargo run &
   curl -s http://127.0.0.1:9091/health   # {"status":"ok"}
   curl -s http://127.0.0.1:8081/health   # connection refused
   ```

7. Коммит:

   ```bash
   git add backend/src/lib.rs backend/src/main.rs
   git commit -m "feat(backend): listen on BACKEND_PORT with default 8081"
   ```

## Файлы

- Modify: `backend/src/lib.rs` (функция `bind_addr` + `#[cfg(test)]`-модуль),
  `backend/src/main.rs` (чтение `BACKEND_PORT`, вызов `bind_addr`).
- Не трогать: `frontend/**`, `openapi/**`, `contracts/**`, CI-workflows.

## Проверка (real-run)

| # | Команда                                                  | Ожидаемый результат                                     |
| - | -------------------------------------------------------- | ------------------------------------------------------- |
| 1 | `cd backend && cargo test --lib`                          | PASS, 2 теста (`default_is_loopback_8081`, `backend_port_overrides_default`) |
| 2 | `make test`                                              | exit 0; все прежние тесты зелёные                       |
| 3 | `make lint`                                              | exit 0 (fmt + clippy `-D warnings`)                     |
| 4 | `BACKEND_PORT=9091 cargo run` + `curl /health`            | 200 на 9091, отказ на 8081                              |
| 5 | `git status --porcelain` после коммита                   | пусто                                                   |

## Коммит

Выполняет агент в ветке `task-6`: `feat(backend): listen on BACKEND_PORT with
default 8081`. Один шаг — один коммит.

## Предусловия следующих шагов

Шаг 2 меняет только frontend (`next.config.ts`, `api-config.ts`, тесты) —
от шага 1 требует только зелёного baseline. Шаг 3 (entrypoint) использует
факт «backend читает `BACKEND_PORT`, дефолт 8081».

## Источники

- real-run (чтение файлов, 2026-10-01): `backend/src/main.rs` (жёсткий
  `127.0.0.1:8081`), `backend/src/lib.rs` (структура, `app()`/`app_with_state`),
  `cargo test` baseline (lib-секция — 0 тестов).
- real-run (shell, 2026-10-01): `make test`/`make lint` exit 0 (baseline).
- code-reading: чистая функция вместо чтения env внутри тестов — чтобы
  юнит-тесты не зависели от глобального состояния и параллельного запуска.
