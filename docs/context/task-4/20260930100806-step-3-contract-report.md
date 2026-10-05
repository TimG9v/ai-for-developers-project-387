# step-3-contract-report

- Дата: 2026-09-30 10:08
- Ветка: task-4 (рабочая)
- Шаг: отчёт о выполнении шага 3 плана — TypeSpec-контракт и OpenAPI

Исходный план: `20260930074841-step-3-typespec-openapi.md`; решения карты —
закрытые билеты #3–#9, генератор серверных типов — #10 (typify).

## Что сделано

1. `contracts/` — изолированный пакет тулчейна контракта: `npm init` +
   devDeps `@typespec/compiler`, `@typespec/http`, `@typespec/rest`,
   `@typespec/openapi3` (все 1.16.0, ставились одной командой, real-run).
2. `contracts/main.tsp` (71 строка): модели EventType / Slot / Booking;
   ошибки ConflictError (409) и NotFoundError (404); интерфейсы
   EventTypes / Slots / Bookings (list + create); `@service` с заголовком
   «Calendar — API записи на звонок»; всё внутри namespace `Booking`.
3. `contracts/tspconfig.yaml`: эмиттер `@typespec/openapi3` →
   `{project-root}/../openapi`, `openapi.yaml`.
4. `contracts/.gitignore`: node_modules вне репо.
5. `openapi/openapi.yaml` (167 строк) — сгенерированный артефакт, коммитится.

## Решения контракта (трассировка к карте)

| Решение карты                       | В контракте                                                  |
| ----------------------------------- | ------------------------------------------------------------ |
| #3 Тип встречи, слот длится по типу | `EventType.durationMinutes`, `Slot.eventTypeId` (required)   |
| #4 Слоты создаёт владелец           | `POST /slots` (без охраны — ADR 0002/#5)                     |
| #5 Owner-API без охраны             | Никаких security-scheme в контракте                          |
| #6 Гость обязателен                 | `guestName`, `guestEmail` — required, валидация — бекенд     |
| #7 Слот атомарен, 409               | `Bookings_create` → 200/404/409, doc «не более одной записи» |
| #8 Окно 14 дней                     | `@doc` на `Slots.list` + query-фильтр `eventTypeId`          |

## Отклонение от скелета плана

`Slot.eventTypeId` (обязательное поле + `@query eventTypeId?: string` в list):
скелет плана связи не имел, но решение #3 («длительность слота определяет его
тип встречи») и сценарий гостя из descripton.md («выбирает тип → календарь →
слот») требуют typed-слотов. Пользователю предложено вето: правка одной строки
+ перегенерация.

## Errata к research #10 (подтверждено компиляцией)

- `emitter-output-dir` — интерполяция `{project-root}`, не `${project-root}`;
- `info` — невалидный ключ tspconfig в 1.16 (invalid-schema); заголовок —
  декоратор `@service` над namespace;
- объектные литералы в декораторах — `#{...}`, не `{...}` (expect-value).

## Проверка (real-run 2026-09-30)

| # | Команда                                           | Результат                      |
| - | ------------------------------------------------- | ------------------------------ |
| 1 | `npx tsp compile .`                               | exit 0, Compilation successful |
| 2 | `grep -c operationId openapi/openapi.yaml`        | 6 (план: ≥6)                   |
| 3 | `grep -n "'404'\|'409'"`                          | обе ошибки в Bookings_create   |
| 4 | required гостя и eventTypeId                      | строки 111–112, 150–153 спеки  |
| 5 | `openapi: 3.0.0` в шапке                          | совместимость с typify (#10)   |
| 6 | повторный `tsp compile` + `git diff` по артефакту | пусто — детерминизм            |
| 7 | `head -4 openapi.yaml`                            | title применился из @service   |

## Что не проверено

- Потребители контракта: SDK (шаг 4) и typify (шаг 5) — следующая ступень
  проверки контракта на практике.

## Коммит (делает пользователь)

```bash
git add contracts openapi
git commit -m "feat(contracts): add TypeSpec contract with OpenAPI generation"
```

Примечание: правки `CONTEXT.md`/ADR 0002 из чартинга пользователем уже
закоммичены (git status на 10:07 показывает только contracts/ и openapi/).

## Следующие шаги

Шаг 4 — клиентский SDK (`@hey-api/openapi-ts`) + тест использования;
шаг 5 — typify в backend (путь подтверждён #10, errata учтена);
шаг 6 — `make generate`; билет #11 карты остаётся на сессию `/to-spec`.

## Источники

- real-run (shell, 2026-09-30): npm install, tsp compile (4 прогона: 3 ошибки
  синтаксиса исправлены по тексту диагностики), grep-проверки спеки,
  `git diff --stat` (детерминизм), `git status --porcelain`.
- real-run (чтение): `openapi/openapi.yaml` (полностью, 167 строк);
  текст ошибок компилятора (config-path-absolute, invalid-schema,
  expect-value).
- code-reading: решения закрытых билетов карты #3–#9, #10 (через контекст
  сессии чартинга и research-отчёт в комментарии #10).
