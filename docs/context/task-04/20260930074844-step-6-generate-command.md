# step-6-generate-command

- Дата: 2026-09-30 07:48
- Ветка: task-4 (рабочая)
- Шаг: план шага 6/8 — вся цепочка генерации одной командой

Исходный план: `20260930074838-implementation-plan.md`.

## Цель

Требование `descripton.md`: «соберите команды генерации так, чтобы всю цепочку
можно было повторить одной командой» и «генерация повторяется одной командой,
сгенерированное руками не правится». В репозитории есть `make generate`:
TypeSpec → OpenAPI → клиентский SDK → серверные типы; повторный прогон на чистом
дереве не меняет ни одного файла.

## Предусловия

1. Шаги 3–5 выполнены: `npx tsp compile .` (contracts), `npm run generate:client`
   (frontend), `cargo build` c build.rs (backend) — каждый зелёный по отдельности.
2. Дерево чистое перед проверкой детерминизма.

## Действия

1. `Makefile` в корне — цель и порядок (контракт раньше консьюмеров):

   ```makefile
   .PHONY: generate

   generate:
   	cd contracts && npx tsp compile .
   	cd frontend && npm run generate:client
   	cd backend && cargo build
   ```

   (табуляция в рецептах обязательна; `generate` дополнить в `.PHONY` в шапке.)

2. Зафиксировать правило «сгенерированное руками не правится» в `AGENTS.md`:
   таблицу Команды дополнить строкой `generate — make generate — перегенерация
   OpenAPI, SDK и серверных типов из TypeSpec-контракта; артефакты руками не
   править`.

3. Прогнать цепочку на чистом дереве и проверить детерминизм:

   ```bash
   make generate && git status --porcelain   # пусто
   make generate && git status --porcelain   # снова пусто
   ```

   Нестабильный вывод генераторов (даты, сортировка ключей) чинить в конфиге
   генератора, не откатываясь на ручные правки артефактов.

## Файлы

- Modify: `Makefile` (цель generate + .PHONY)
- Modify: `AGENTS.md` (таблица Команды)

## Проверка (real-run)

| # | Команда                                        | Ожидаемый результат                          |
| - | ---------------------------------------------- | -------------------------------------------- |
| 1 | `make generate`                                | exit 0: tsp → openapi-ts → cargo build       |
| 2 | `git status --porcelain` после первого прогона | пусто (артефакты уже закоммичены шагами 3–5) |
| 3 | второй `make generate` + `git status`          | пусто — детерминизм                          |
| 4 | `make test`                                    | тесты обоих приложений зелёные               |
| 5 | `make lint`                                    | fmt/clippy/eslint чистые                     |

## Коммит (делает пользователь)

```bash
git add Makefile AGENTS.md
git commit -m "ci: add single generate command for contract pipeline"
```

## Предусловия следующих шагов

Шаги 7–8 живут в трекере и опираются на решения карты; команда `make generate`
упоминается в спеке как способ консистентного обновления контракта при правках.

## Источники

- real-run (чтение файлов, 2026-09-30): `Makefile` (существующие цели dev/test/
  lint — стиль табов и .PHONY), `AGENTS.md` (таблица Команды), Makefile-цели
  `lint-backend`/`lint-frontend` — основа п. 5 Проверки.
- code-reading: порядок цепочки — из цепочки descripton.md (TypeSpec → OpenAPI →
  SDK/серверные артефакты).
