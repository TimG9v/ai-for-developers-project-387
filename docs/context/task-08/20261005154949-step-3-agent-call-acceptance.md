# step-3-agent-call-acceptance

- Дата: 2026-10-05 15:49
- Ветка: `main` (после шагов 1–2); коммитов кода шаг не даёт
- Шаг: план шага 3/3 — issue, вызов агента `/oc explain this issue`,
  разбор прогона Actions, приёмка цикла

Исходный план: `20261005154949-implementation-plan.md`.

## Цель

Существует issue, на комментарий в котором агент OpenCode ответил;
соответствующий прогон виден во вкладке Actions с длительностью и логами
шага агента; таблица приёмки всех шести требований спеки заполнена
real-run-доказательствами.

## Interfaces

- Consumes: воркфлоу `opencode` в `main` (шаг 1); установленный App и
  секрет `OPENCODE_API_KEY` (шаг 2); пункт «Баг 1: дубликаты сущностей…»
  плана развития `docs/context/task-7pre/20261005142605-development-plan.md`
  (текст issue — формулировка без изменения сути, конвенция development-plan).
- Produces: issue с ответом агента; прогон воркфлоу `opencode` в Actions;
  заполненная таблица приёмки — основа отчёта цикла.

## Предусловия (real-run перед стартом)

- [ ] 1. Воркфлоу в `main` и секрет на месте:

  ```bash
  git fetch && git show origin/main:.github/workflows/opencode.yml >/dev/null && echo WF-OK
  gh secret list | grep OPENCODE_API_KEY
  ```

  Ожидание: `WF-OK`; строка `OPENCODE_API_KEY` в списке секретов.
  Отсутствует хотя бы одно — вернуться в шаг 1/2, вызов ронять нельзя.

## Действия

- [ ] 1. Создать issue из пункта «Баг 1» плана развития (жалоба
  пользователя, без готового решения):

  ```bash
  gh issue create \
    --title "Дубликаты типа встречи при повторной отправке формы создания" \
    --body "После повторной отправки формы создания (сбой сети, повтор
  запроса) тип встречи появляется в списке дважды: сервис молча принимает
  обе копии. Дублей может быть сколько угодно, они неотличимы друг от
  друга, и владелец не может понять, на какой из них ссылаются слоты и
  записи.

  Источник: docs/context/task-7pre/20261005142605-development-plan.md,
  «Баг 1» (подтверждён real-run 2026-10-05 на публичном деплое)." \
    --label "bug"
  ```

  Ожидание: URL issue (номер N). Если метки `bug` нет — создать:
  `gh label create bug --color d73a4a` и повторить.

- [ ] 2. Позвать агента командой из описания задачи:

  ```bash
  gh issue comment N --body "/oc explain this issue"
  ```

  Ожидание: комментарий создан; воркфлоу `opencode` стартует автоматически
  (событие `issue_comment`, `types: [created]`, условие `/oc` выполнено).

- [ ] 3. Дождаться завершения прогона (обычно 1–5 минут; при free-модели
  возможен дневной лимит — см. Review Focus):

  ```bash
  gh run list --workflow=opencode.yml --limit 1
  sleep 60 && gh run list --workflow=opencode.yml --limit 1
  ```

  Ожидание: строка прогона с `completed` и `success`; статус `in_progress`
  — повторить просмотр, не мешать прогону.

- [ ] 4. Проверить ответ агента в issue:

  ```bash
  gh issue view N --comments
  ```

  Ожидание: после комментария с `/oc` появился ответ от имени приложения
  (GitHub App identity), содержательно разбирающий issue.

- [ ] 5. Разбор прогона (задача 5 описания): длительность и логи шага
  агента:

  ```bash
  gh run view $(gh run list --workflow=opencode.yml --limit 1 --json databaseId --jq '.[0].databaseId')
  gh run view $(gh run list --workflow=opencode.yml --limit 1 --json databaseId --jq '.[0].databaseId') --log | grep -A 40 "Run OpenCode"
  ```

  Ожидание (зафиксировать в отчёте): длительность прогона; в логах шага
  «Run OpenCode» — старт агента, указанная модель `opencode/glm-5.3-flash`,
  отсутствие секретов в выводе (если ключ появился в логе — инцидент:
  отзыв ключа у куратора, новый секрет, разбор отдельным контекстом).

- [ ] 6. Приёмка цикла — таблица ниже, каждая строка real-run:

| # | Требование (description.md)                                    | Доказательство                                                                |
| - | -------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| 1 | GitHub App установлен на репозиторий                           | подтверждение пользователя (шаг 2) + успешный OIDC-прогон этого шага          |
| 2 | Воркфлоу в `.github/workflows/` замержен в `main`              | `git show origin/main:.github/workflows/opencode.yml`, merge-коммит PR шага 1 |
| 3 | События `types: [created]`, работа — условием по команде       | разбор в шаге 1 + grep; живое подтверждение — прогон от комментария           |
| 4 | Права `id-token: write`, checkout `persist-credentials: false` | разбор в шаге 1 + grep шага 1                                                 |
| 5 | Ключ в GitHub Secrets, в файлах репозитория его нет            | `gh secret list` (шаг 2) + git grep шага 2 + лог прогона без ключа            |
| 6 | Issue с ответом агента и прогоном в Actions                    | issue N (ответ агента) + `gh run list --workflow=opencode.yml`                |

- [ ] 7. Финальный статус CI на `main` не деградировал:

  ```bash
  gh run list --branch main --limit 6
  ```

  Ожидание: backend-ci, frontend-ci, hexlet-check, release-please — success
  на последнем коммите `main`; opencode в списке push-прогонов отсутствует
  (событийный воркфлоу).

## Файлы

- Create: ничего в коде; отчёт цикла — новый контекст-файл в
  `docs/context/tmp/task-8/` (например `<TIMESTAMP>-cycle-report.md`)
  с таблицей приёмки, номером issue, id прогона, длительностью и выжимкой
  логов.
- Modify: ничего.
- Не трогать: код, контракт, существующие воркфлоу, README, AGENTS.md.

## Проверка (real-run)

| # | Команда                                                                             | Ожидаемый результат                                         |
| - | ----------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| 1 | `gh api repos/TimG9v/ai-for-developers-project-387/issues/N/comments --jq 'length'` | `2` или больше (команда + ответ агента)                     |
| 2 | `gh run list --workflow=opencode.yml --limit 1`                                     | `completed`, `success`                                      |
| 3 | `gh run view <id>`                                                                  | длительность ≠ 0; оба шага (checkout, Run OpenCode) success |
| 4 | grep модели из действия 5 (glm-5.3-flash в логе)                                    | ≥ 1 совпадение                                              |
| 5 | grep ключа из действия 5 (oc-… в логе)                                              | 0 совпадений значения                                       |
| 6 | `gh issue view N --comments`                                                        | ответ агента после комментария с `/oc`                      |
| 7 | `gh run list --branch main --limit 6`                                               | предсуществующие workflows — success                        |

## Отрицательные проверки (один прогон, дёшево)

- Комментарий без команды не должен будить агента: на любом issue
  оставить обычный комментарий (без `/oc`) — прогон `opencode` не
  появляется в `gh run list`. Проверять на новом комментарии, не на
  issue N (чтобы не плодить прогоны).

## Коммит

Шаг без коммитов кода. Контекст-файл отчёта коммитится по решению
пользователя в общий `docs:`-коммит цикла (вместе с судьбой правки
`docs/context/task-7pre/description.md` и переездом контекстов).

## Результат шага (description.md)

Агент подключён к репозиторию: воркфлоу лежит в основной ветке, ключ в
секретах, вызов из issue-комментария подтверждён ответом агента и
прогоном в Actions.

## Источники

- real-run (fetch, 2026-10-05): opencode.ai/docs/github — пример
  «Explain an issue» (`/opencode explain this issue`), семантика ответа
  агента в issue; opencode.ai/docs/zen — дневной лимит бесплатных моделей,
  вкладка Logs для расхода; help.hexlet.io/ai/opencode — лимит, повтор
  прогона при ошибке лимита.
- real-run (shell, 2026-10-05): `gh issue list` (issue #1 OPEN — номера
  новых issue пойдут дальше), `gh run list` (формат вывода), конвенции
  `docs/agents/issue-tracker.md` (gh issue create/comment/view).
- code-reading: текст «Баг 1» — из development-plan task-7pre (там
  заявлено «оформляются как issue… без изменения формулировок сути»);
  ожидаемая длительность — порядок величины агентов в Actions.
