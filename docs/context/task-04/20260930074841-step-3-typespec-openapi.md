# step-3-typespec-openapi

- Дата: 2026-09-30 07:48
- Ветка: task-4 (рабочая)
- Шаг: план шага 3/8 — TypeSpec-контракт и генерация OpenAPI

Исходный план: `20260930074838-implementation-plan.md`.

## Цель

В репозитории лежит TypeSpec-контракт, покрывающий сценарии владельца и гостя
(`descripton.md`, требования 4–5), OpenAPI-спецификация генерируется из него
командой и коммитится как артефакт. Сгенерированное руками не правится: правится
`main.tsp`, остальное — продукт компиляции.

## Предусловия

1. Шаг 2 закрыт: решения карты, от которых зависит контракт, приняты (термины,
   модель слотов, данные гостя, конфликт, окно 14 дней).
2. Node.js по `.nvmrc` (real-run: файл есть в корне).
3. Контракт сверён с Decisions so far карты: при расхождении выигрывает карта,
   а не черновики.

## Действия

1. Каталог контракта с изолированным package.json (не трогает frontend):

   ```bash
   mkdir -p contracts && cd contracts
   npm init -y
   npm install --save-dev @typespec/compiler @typespec/openapi3
   ```

2. `contracts/main.tsp` — модели и эндпоинты **по решениям карты**. Рекомендательный
   скелет (имена моделей от descripton.md; термины и поля сверить с картой):

   ```typespec
   import "@typespec/http";

   using TypeSpec.Http;

   model EventType {
     id: string;
     title: string;
     @doc("Описание типа встречи") description?: string;
     durationMinutes: int32;
   }

   model Slot {
     id: string;
     startDateTime: utcDateTime;
     endDateTime: utcDateTime;
   }

   model Booking {
     id: string;
     slotId: string;
     guestName: string;
     guestEmail: string;
     createdAt: utcDateTime;
   }

   @route("/event-types") interface EventTypes {
     list(): EventType[];
     @post create(@body body: EventType): EventType;
   }

   @route("/slots") interface Slots {
     @doc("Свободные слоты: окно 14 дней от текущей даты")
     list(): Slot[];
     @post create(@body body: Slot): Slot;
   }

   @route("/bookings") interface Bookings {
     @doc("Все записи; ракурс владельца") list(): Booking[];
     @post create(@body body: Booking): Booking | ConflictError | NotFoundError;
   }

   @doc("Слот уже занят") model ConflictError { @statusCode statusCode: 409; }
   @doc("Слот или тип не найдены") model NotFoundError { @statusCode statusCode: 404; }
   ```

   Отличия от черновика `20260930100002` — сознательные и требуют подтверждения
   картой: `Booking` не вкладывает `EventType` (выводимое поле), гость обязателен,
   ошибки 409/404 в контракте (в черновике их не было), `isAvailable` убран
   (атомарный слот: занятость = наличие записи). Если карта решила иначе — править
   скелет по ответам, не по черновику.

3. `contracts/tspconfig.yaml`:

   ```yaml
   emit:
     - "@typespec/openapi3"
   options:
     "@typespec/openapi3":
       output-file: ../openapi/openapi.yaml
   ```

4. Сгенерировать и закоммитить артефакт:

   ```bash
   cd contracts && npx tsp compile .
   ```

   `openapi/openapi.yaml` коммитится (ревью контракта по diff), при этом любой
   может перегенерировать его одной командой.

5. Сверка покрытия сценариев (`descripton.md`): владелец — создать тип встречи,
   список записей всех типов; гость — список типов, календарь слотов на 14 дней,
   создание записи; занятость — 409. Каждому сценарию — свой operation в контракте.

## Файлы

- Create: `contracts/package.json`, `contracts/tspconfig.yaml`, `contracts/main.tsp`
- Create (генерат): `openapi/openapi.yaml`
- Create: `contracts/node_modules` — в `contracts/.gitignore`

## Проверка (real-run)

| # | Команда                                       | Ожидаемый результат                   |
| - | --------------------------------------------- | ------------------------------------- |
| 1 | `cd contracts && npx tsp compile .`           | exit 0, без diagnostics error         |
| 2 | `ls openapi/openapi.yaml`                     | файл создан                           |
| 3 | `grep -c "operationId" openapi/openapi.yaml`  | ≥ 6 (list/create по трём интерфейсам) |
| 4 | `grep -A2 "409" openapi/openapi.yaml \| head` | ConflictError присутствует            |
| 5 | `npx tsp compile . && git status --porcelain` | повторный прогон не меняет артефакт   |

## Коммит (делает пользователь)

```bash
git add contracts openapi
git commit -m "feat(contracts): add TypeSpec contract with OpenAPI generation"
```

## Предусловия следующих шагов

Шаг 4 читает `openapi/openapi.yaml`; шаг 5 — тот же файл со стороны backend;
шаг 6 оборачивает `npx tsp compile .` в `make generate`.

## Источники

- real-run (чтение файлов, 2026-09-30): `context/tmp/task-4/descripton.md`
  (пример TypeSpec→OpenAPI, Ссылки: TypeSpec, openapi-ts, OpenAPI Generator);
  `20260930100002-step-2-api-design.md` (черновик контракта — материал).
- code-reading: `docs/agents/issue-tracker.md` — не задействован; выбор emitter
  `@typespec/openapi3` — рекомендация по Ссылкам descripton.md, подтверждается на
  шаге реальной установкой.
