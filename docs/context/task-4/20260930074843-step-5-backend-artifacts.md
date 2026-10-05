# step-5-backend-artifacts

- Дата: 2026-09-30 07:48
- Ветка: task-4 (рабочая)
- Шаг: план шага 5/8 — серверные артефакты из спецификации

Исходный план: `20260930074838-implementation-plan.md`.

## Цель

Из `openapi/openapi.yaml` генерируются серверные артефакты backend — как минимум
типы контракта (состав зависит от фреймворка, `descripton.md`, требование 7);
`cargo test` зелёный, сгенерированный код не редактируется руками.

## Предусловия

1. Шаг 3 выполнен: `openapi/openapi.yaml` существует.
2. Research-билет карты о генераторе закрыт (шаг 2, билет 7). Варианты с
   trade-offs — вопрос билета:

| Вариант                                            | Плюсы                                          | Минусы                              |
| -------------------------------------------------- | ---------------------------------------------- | ----------------------------------- |
| B. `typify` (build.rs, rust-native) — рекомендация | без внешних рантайм-зависимостей, типы в crate | валидацию и маршруты пишем сами     |
| A. `openapi-generator-cli` (`-g rust`)             | много артефактов сразу                         | Java-зависимость в тулчейне проекта |

   Команды ниже — для рекомендованного B; при выборе A шаг перепланируется по
   ответу research-билета (генератор ставится один раз, команда генерации
   фиксируется так же, как в шаге 4).

## Действия (рекомендованный путь B)

1. Зависимости сборки в `backend/Cargo.toml`:

   ```toml
   [build-dependencies]
   typify = "0.4"
   serde_json = "1"
   serde_yaml = "0.9"
   ```

   Версии сверить на crates.io при установке (real-run `cargo add`).

2. `backend/build.rs` — чтение спеки, извлечение `components.schemas`, генерация
   типов в `OUT_DIR`:

   ```rust
   use std::env;
   use std::fs;
   use std::path::PathBuf;

   fn main() {
       let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
           .join("../openapi/openapi.yaml");
       println!("cargo:rerun-if-changed={}", spec_path.display());

       let spec: serde_yaml::Value = serde_yaml::from_str(
           &fs::read_to_string(&spec_path).expect("read openapi.yaml"),
       )
       .expect("parse openapi.yaml");
       let schemas = spec["components"]["schemas"].clone();

       let mut settings = typify::Settings::new();
       let space = settings
           .with_struct_builder(true)
           .load(schemas)
           .expect("schemas load")
           .to_stream()
           .to_string();

       let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("api_types.rs");
       fs::write(&out, space).expect("write api_types.rs");
   }
   ```

   Точная поверхность API typify (`Settings`, builder-флаги) сверяется с docs.rs
   при написании — скелет показывает форму, не дословный API.

3. Мост в crate: `backend/src/api.rs`:

   ```rust
   pub mod api_types {
       include!(concat!(env!("OUT_DIR"), "/api_types.rs"));
   }
   ```

   и `mod api;` в `backend/src/lib.rs`.

4. Тест использования (проверка, что артефакты собираются и типизированы) —
   `backend/tests/api_types.rs` или unit-тест в `lib.rs`:

   ```rust
   #[test]
   fn event_type_roundtrip() {
       use serde::Deserialize;
       let raw = r#"{"id":"et1","title":"Звонок","durationMinutes":30}"#;
       let parsed = backend::api::api_types::EventType::deserialize(
           &mut serde_json::Deserializer::from_str(raw),
       )
       .expect("parse EventType from contract shape");
       assert_eq!(parsed.duration_minutes, 30);
   }
   ```

   Имя типа и стиль полей (`duration_minutes` vs `durationMinutes`) — как решит
   typify (serde-переименование); тест правится по факту генерации, контракт —
   не правится.

5. Прогнать (real-run): `cd backend && cargo build && cargo test`.

## Файлы

- Create: `backend/build.rs`, `backend/src/api.rs`
- Modify: `backend/Cargo.toml` (build-dependencies), `backend/src/lib.rs` (`mod api`)
- Modify: `backend/tests/` (тест использования)
- Генерат living in `OUT_DIR` — в git не попадает; источником правды остаётся
  `openapi.yaml` (правка типов — только через контракт).

## Проверка (real-run)

| # | Команда                                      | Ожидаемый результат                      |
| - | -------------------------------------------- | ---------------------------------------- |
| 1 | `cd backend && cargo build`                  | exit 0, build.rs читает спеку            |
| 2 | `cargo test`                                 | тест EventType зелёный, smoke зелёный    |
| 3 | `cargo clippy --all-targets -- -D warnings`  | чисто (правило lint-backend из Makefile) |
| 4 | touch ../openapi/openapi.yaml && cargo build | пересборка по rerun-if-changed           |

## Коммит (делает пользователь)

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/build.rs backend/src backend/tests
git commit -m "feat(backend): generate API types from OpenAPI spec"
```

## Предусловия следующих шагов

Шаг 6 добавляет backend-сборку в цепочку `make generate` (генерация типов
происходит при `cargo build` через build.rs).

## Источники

- real-run (чтение файлов, 2026-09-30): `context/tmp/task-4/descripton.md`
  (Ссылки: OpenAPI Generator; требование 7 — «состав зависит от фреймворка»);
  `20260930100004-step-4-backend-structure.md` (черновик слоёв backend — материал
  для тикетов, не для этого шага).
- code-reading: скелет build.rs — паттерн typify (Oxide), дословная поверхность
  API подтверждается на шаге (п. 1–2 Проверки).
