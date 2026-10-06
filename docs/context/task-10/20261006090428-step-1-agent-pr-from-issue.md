# step-1-agent-pr-from-issue

- Дата: 2026-10-06 09:04 (+05)
- Ветка: без рабочей ветки — коммитов кода и файлов плана нет; вся
  работа через GitHub (issue #7, PR агента)
- Шаг: план шага 1/4 — `/oc`-команда в issue #7 → ветка `opencode/*` +
  PR от агента; проверка PR

Исходный план: `20261006090428-implementation-plan.md`.

## Цель

Существует открытый PR, созданный агентом: ветка `opencode/*`, тело с
описанием изменений и ссылкой на #7 (`Closes #7`), коммиты по
Conventional Commits (минимум один `feat:`), CI на PR зелёный.

## Interfaces

- Consumes: постановка и план исправления в треде #7 (task-9);
  `opencode.yml` — канал `issue_comment` (`/oc`-гейт,
  `id-token: write`, `zai-coding-plan/glm-5.3-flash`); AGENTS.md —
  правила репо, агент читает их из чекаута.
- Produces: открытый PR агента (ветка `opencode/*`) — единственный вход
  шага 3 (ревью-цикл) и шага 4 (мерж). Точного имени ветки агент
  выбирает сам; шаги 3–4 берут его из `gh pr list`.

## Предусловия (real-run перед стартом)

- [ ] 1. `main` свежий, дерево чистое:

  ```bash
  git checkout main && git pull && git status --porcelain
  ```

  Ожидание: `status` пуст.

- [ ] 2. #7 открыта, постановка на месте:

  ```bash
  gh issue view 7 --json state,title
  gh issue view 7 --comments | grep -c "Что считается исправлением"
  ```

  Ожидание: `state: OPEN`; grep ≥ 1.

## Действия

- [ ] 1. Оставить в #7 команду (постит пользователь; точный текст):

  ```text
  /oc подготовь pull request с исправлением по плану и постановке выше.
  Ветку и pull request создай сам. В описании PR: краткое описание
  изменений и «Closes #7». Коммиты — строго по Conventional Commits
  (см. AGENTS.md), минимум один feat:. Сгенерированные файлы руками
  не править — только перегенерация make generate из contracts/main.tsp.
  ```

  Почему требования дублируются в команде: агент читает тред и
  AGENTS.md, но явная формулировка делает исход прогона проверяемым —
  `Closes #7` закрывает требование 1 спеки (связь с issue), `feat:` —
  условие работы release-please (шаг 4), запрет ручных правок
  генерации — критерий авторевью (шаг 2).

- [ ] 2. Дождаться завершения прогона:

  ```bash
  gh run list --workflow opencode --limit 3
  ```

  Ожидание: верхний прогон, event `issue_comment`, `success`. Прогон
  большой (реализация фичи из 7 шагов плана) — минуты, не секунды;
  не прерывать и не добавлять новые команды в тред, пока он идёт.

- [ ] 3. Найти PR агента:

  ```bash
  gh pr list --state open --json number,headRefName,title,url
  ```

  Ожидание: открытый PR из ветки `opencode/*`.

  **Стоп-протокол** (прогон `success`, а PR нет): прочитать последний
  комментарий бота в #7 — агент мог отчитаться ошибкой (например,
  прав приложения на создание ветки/PR). Команду вслепую не повторять;
  варианты действий — открытый вопрос 3 плана (решение пользователя).

- [ ] 4. Проверить PR (задача 2 описания: описание, изменения, связь
  с issue):

  ```bash
  gh pr view <N> --json title,body,headRefName
  # тело: описание изменений + «Closes #7»

  gh pr view <N> --json commits -q '.commits[].messageHeadline'
  # все headlines по Conventional Commits; минимум один feat:

  gh pr diff <N> --stat
  # объём: contracts/ + backend/ + frontend/; сгенерированное —
  # только как перегенерация, без точечных ручных правок

  gh pr checks <N>
  # backend-ci, frontend-ci — success
  ```

- [ ] 5. Связь с issue (требование 1): `Closes #7` в теле связывает PR
  с issue (секция Development в UI; при мерже issue закроется
  автоматически). Проверка:

  ```bash
  gh pr view <N> --json body | grep -c "Closes #7"
  ```

  Ожидание: ≥ 1.

## Файлы

- Create: ничего в репо — PR, ветку и коммиты создаёт агент
  (`opencode/*`).
- Modify: ничего; предсуществующие файлы не трогаются.

## Проверка (real-run)

| # | Команда                                     | Ожидаемый результат                           |
| - | ------------------------------------------- | --------------------------------------------- |
| 1 | `gh run list --workflow opencode --limit 1` | `success`, event `issue_comment`              |
| 2 | `gh pr list --state open`                   | PR из ветки `opencode/*`                      |
| 3 | `gh pr view <N> --json body`                | содержит `Closes #7` и описание изменений     |
| 4 | `gh pr view <N> --json commits`             | headlines по Conventional Commits, есть feat: |
| 5 | `gh pr checks <N>`                          | все проверки зелёные                          |

## Отрицательные проверки

- В `main` нет коммитов агента — правки только в ветке PR:

  ```bash
  git fetch && git log --oneline origin/main -3
  ```

  Ожидание: без `opencode`-коммитов.

- Сгенерированные файлы в PR изменены только перегенерацией (состав и
  объём диффа по `frontend/src/client/` оценивается на ревью шага 3;
  здесь — грубая проверка `gh pr diff <N> --stat`).

## Коммит

Коммитов шага нет: и PR, и коммиты — работа агента. Контекст-файлы
плана — `docs:`-коммит на ветке `task-10` в шаге 2 (по решению
пользователя, образец task-9 `e263a7f`).

## Результат шага (description.md)

Есть pull request, созданный агентом по разобранной задаче (#7),
связанный с исходным issue (задачи 1–2 описания, требование 1).

## Источники

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — «Fix an
  issue»: `/opencode fix this` → агент создаёт новую ветку, реализует
  изменения и открывает PR (core-поведение); Supported Events
  `issue_comment`: агент читает весь тред комментариев.
- real-run (shell, 2026-10-06): `gh issue view 7` (постановка на
  месте); `.github/workflows/opencode.yml` (связка model/env,
  `/oc`-гейт); `gh workflow list` (opencode active, id 375442803).
- real-run (чтение файлов, 2026-10-06): task-9 cycle report (#7 — план
  и постановка; #9 — 4 открытых вопроса агента, потому не #9).
- code-reading: стоп-протокол — из инцидента task-8 (ошибки бота в
  треде вместо результата) и perm-паттерна; требование `feat:` —
  семантика release-please (`chore:` не даёт релиз-бампа); запрет
  ручных правок генерации — AGENTS.md.
