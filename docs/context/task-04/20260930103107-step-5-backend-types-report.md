# step-5-backend-types-report

- Дата: 2026-09-30 10:31
- Ветка: task-4 (рабочая)
- Шаг: отчёт о выполнении шага 5 плана — серверные артефакты из спецификации

Исходный план: `20260930074843-step-5-backend-artifacts.md`; генератор выбран
research-билетом [#10](https://github.com/TimG9v/ai-for-developers-project-386/issues/10)
(typify 0.8 через build.rs, путь B).

## Что сделано

1. `contracts/tspconfig.yaml`: `file-type: [yaml, json]` + `output-file:
   "openapi.{file-type}"` — YAML для ревью, JSON для build.rs (без serde_yaml:
   архивирован, cargo audit в security.yml его не одобрит).
2. `backend/build.rs`: читает `../openapi/openapi.json` → `components.schemas`
   → `Vec<(String, schemars::schema::Schema)>` → `TypeSpaceSettings` (с
   билдерами) → `TypeSpace::add_ref_types` → `to_stream` → `prettyplease` →
   `OUT_DIR/openapi_types.rs` с маркером `@generated`; `rerun-if-changed` на спеку.
3. `backend/src/api.rs` + `pub mod api` в `lib.rs`: мост `include!` генерата.
4. `backend/Cargo.toml`: build-deps typify/schemars/syn/prettyplease/serde_json;
   deps chrono(serde); dev-deps serde_json (тестам нужен парсер).
5. `backend/tests/api_types.rs`: 3 теста формы контракта.
6. `openapi/openapi.json` — новый артефакт эмиттера, коммитится.

## Отклонение от скелета плана (запланированное)

Скелет build.rs в плане (`typify::Settings`) был помечен «сверить с docs.rs» —
реальный API (по research #10 и факту): `TypeSpaceSettings::default()` +
`with_struct_builder(true)` + `TypeSpace::new(&settings)` + `add_ref_types`.

## Errata к research #10 (найдено фактом)

- `file-type: [...]` работает только с плейсхолдером `{file-type}` в
  `output-file`: без него обе итерации эмиттера пишут в один путь — YAML молча
  перезаписывается JSON (поймано по содержимому `openapi.yaml`; источник —
  `resolveOutputFile` в dist эмиттера: интерполяция фиксированного имени даёт
  один файл на двоих).
- E0716: `with_struct_builder` возвращает `&mut Self` — чейн от временного
  значения не компилируется (скелет плана содержал ровно такую ошибку; план
  оговаривал сверку API).

## Сгенерированное (факт, 386 строк)

snake_case + `#[serde(rename = "...")]`; `description?` →
`Option<String>` + skip_serializing_if; `utcDateTime` → `chrono::DateTime<Utc>`;
`int32` → `i32`; билдеры с `ConversionError` — всё совпало с предсказаниями #10.

## Проверка (real-run 2026-09-30, exit-коды без пайповых искажений)

| # | Команда                                     | Результат                            |
| - | ------------------------------------------- | ------------------------------------ |
| 1 | `npx tsp compile .`                         | exit 0, yaml + json оба              |
| 2 | `cargo build`                               | exit 0, 386 строк генерата           |
| 3 | `cargo test`                                | 3 контрактных + smoke, exit 0        |
| 4 | `cargo clippy --all-targets -- -D warnings` | exit 0                               |
| 5 | touch спеки + `cargo build`                 | «Compiling backend» — rerun работает |
| 6 | перегенерация спеки + `git status`          | yaml байт-в-байт равен коммиту       |
| 7 | `make test` (весь проект)                   | exit 0, бекенд 4 + фронт 10/10       |

## Что не проверено

- Использование типов в axum-хендлерах: хендлеры придут вертикальными тикетами
  после `/to-tickets`; сейчас типы проверены парсингом формы контракта.

## Коммит (делает пользователь)

```bash
git add contracts/tspconfig.yaml openapi/openapi.json backend
git commit -m "feat(backend): generate API types from OpenAPI spec"
```

## Следующие шаги

Шаг 6 — `make generate`: вся цепочка одной командой + правило «руками не
правится» в AGENTS.md. Затем шаги 7–8, билет #11 — к сессии `/to-spec`.

## Источники

- real-run (shell, 2026-09-30): npm/cargo-прогоны с фиксацией exit-кодов,
  чтение сгенерированного файла (386 строк), чтение dist-исходника эмиттера
  (resolveOptions/resolveOutputFile), touch-эксперимент rerun-if-changed,
  `git status`/`git diff` (детерминизм).
- code-reading: API typify 0.8 по research-отчёту #10 (комментарий в билете) —
  подтверждено компиляцией.
