# step-1-baseline-hygiene

- Дата: 2026-10-05 10:25
- Ветка: task-7 (создаётся в этом шаге; исходная — `main` @ `9bdd91f`)
- Шаг: план шага 1/5 — baseline и гигиена унаследованных ссылок на 386

Исходный план: `20261005102500-implementation-plan.md`.

## Цель

Рабочая ветка `task-7` с зелёным baseline (`make test`, `make lint`) и
README, в котором бейдж CI, clone-URL и ссылка на issue указывают на
репозиторий 387, а секция «Публичный деплой» временно не содержит
устаревшего URL 386 (новый URL впишется шаг 5).

## Interfaces

- Consumes: состояние `main` @ `9bdd91f` (чистое, синхронизировано).
- Produces: ветка `task-7` с одним коммитом `docs:`; факт «baseline
  зелёный: 26 backend + 35 frontend тестов, lint exit 0» — предусловие
  всех последующих шагов; issue «no-auth admin mutations» в трекере 387 —
  цель ссылки README.

## Предусловия (real-run перед стартом)

- [ ] 1. Дерево чистое и актуальное:

  ```bash
  git pull && git status --porcelain
  ```

  Ожидание: вывод `git status --porcelain` пустой. Утром 2026-10-05 в
  репо шла параллельная работа (fix package-lock, коммит `9bdd91f`) —
  начинать шаг можно только на чистом дереве.

- [ ] 2. Baseline зелёный:

  ```bash
  make test && make lint
  ```

  Ожидание: exit 0 (на 2026-10-05: 26 backend + 35 frontend тестов;
  3 предсуществующих eslint-предупреждения в `admin-upcoming.test.tsx`
  не блокируют — унаследованы от 386, реестр — task-6 baseline).

- [ ] 3. Красный baseline — стоп: дефект унаследованного кода разбирается
  отдельно (не входит в этот цикл), план цикла не выполняется до зелёного.

## Действия

- [ ] 1. Создать ветку:

  ```bash
  git checkout -b task-7
  ```

- [ ] 2. Создать issue-зеркало ссылки README (в 386 это #23; в трекере 387
  пока пусто):

  ```bash
  gh issue create \
    --title "No-auth admin mutations are publicly reachable (accepted demo compromise)" \
    --body "Перенос смысла issue 23 проекта 386. Админ-мутации
  (POST /api/event-types, POST /api/slots) доступны без авторизации —
  осознанный компромисс учебного демо. Продуктовую функциональность цикл
  task-7 не расширяет; решение — задокументировать в README.
  Источник: docs/context/task-6, шаг 5 (acceptance)." \
    --label "documentation"
  ```

  Ожидание: URL нового issue (номер ≠ 23 — трекер 387 начинался пустым).
  Если метки `documentation` нет — создать: `gh label create documentation
  --color 0075ca` и повторить.

- [ ] 3. Правки `README.md` (4 правки, одного прохода):

  а. Бейдж (стр. 4) — 386 → 387:

  ```markdown
  [![hexlet-check](https://github.com/TimG9v/ai-for-developers-project-387/actions/workflows/hexlet-check.yml/badge.svg)](https://github.com/TimG9v/ai-for-developers-project-387/actions)
  ```

  б. Секция «Публичный деплой» (стр. 13–21) — убрать URL 386, оставить
  структуру с оговоркой:

  ```markdown
  ## Публичный деплой

  Деплой на Render выполняется в шаге 5 текущего цикла (см.
  `docs/context/`); ссылка появится здесь после верификации публичного
  URL. Особенности бесплатного инстанса, которые останутся актуальными:

  - после ~15 минут простоя первый запрос отвечает с задержкой на холодный
    старт (десятки секунд);
  - данные хранятся в памяти процесса — ре-деплой или рестарт контейнера
    обнуляет типы встреч, слоты и записи.
  ```

  Скриншоты (стр. 23–29) не трогать: пути `docs/context/task-6/*.png`
  репозиторионные, файлы в 387 есть.

  в. Clone-инструкция (стр. 41–44) — 386 → 387:

  ```markdown
  ```bash
  git clone https://github.com/TimG9v/ai-for-developers-project-387.git
  cd ai-for-developers-project-387
  ```
  ```

  г. Ссылка на issue в «Известных ограничениях» (стр. 129–132) — на
  созданный в действии 2 issue:

  ```markdown
  - Авторизации нет: админ-мутации (`POST /api/event-types`, `/api/slots`)
    публично доступны — осознанный компромисс учебного демо, решение
    отслеживается в
    [issue #N](https://github.com/TimG9v/ai-for-developers-project-387/issues/N).
  ```

- [ ] 4. Проверить, что упоминаний 386 вне исторических контекстов не
  осталось:

  ```bash
  grep -rn "project-386\|calendar-zvonok" \
    --include="*.md" --include="*.yml" --include="*.json" \
    --include="*.toml" --include="*.ts" --include="*.rs" . \
    | grep -v node_modules | grep -v "docs/context" \
    | grep -v ".next" | grep -v target
  ```

  Ожидание: только `.superpowers/sdd/20261001092929-implementation-plan/
  progress.md` (служебный файл суперпowers-сессии 386, вне README — не
  блокирует приёмку; перенос — решение пользователя, в объём шага не
  входит). README в выдаче отсутствует.

- [ ] 5. Baseline не деградировал:

  ```bash
  make lint && git diff --stat
  ```

  Ожидание: lint exit 0 (eslint прогоняется на неизменённом коде —
  контроль, что правки только в README); в diff только `README.md`.

- [ ] 6. Коммит:

  ```bash
  git add README.md
  git commit -m "docs: point README badge, clone url and issue link to project-387"
  ```

## Файлы

- Create: ничего.
- Modify: `README.md` (4 правки: бейдж, секция деплоя, clone, issue-ссылка).
- Не трогать: `contracts/**`, `openapi/**`, `frontend/**`, `backend/**`,
  `Dockerfile`, `docker/**`, `.github/**`, `docs/context/**` (история).

## Проверка (real-run)

| # | Команда                                              | Ожидаемый результат                                                        |
| - | ---------------------------------------------------- | ---------------------------------------------------------------------------- |
| 1 | `git branch --show-current`                          | `task-7`                                                                     |
| 2 | `make test`                                          | exit 0; 26 backend + 35 frontend тестов                                     |
| 3 | `make lint`                                          | exit 0 (3 известных eslint-предупреждения не блокируют)                      |
| 4 | grep из действия 4                                   | README отсутствует в выдаче; остаток — только `.superpowers/sdd/…/progress.md` |
| 5 | `gh issue view N --json state --jq .state`           | `OPEN`                                                                       |
| 6 | `git status --porcelain` после коммита               | пусто                                                                        |

## Коммит

Выполняет агент в ветке `task-7`: `docs: point README badge, clone url and
issue link to project-387`. Один шаг — один коммит.

## Предусловия следующих шагов

Шаг 2 требует от шага 1: ветку `task-7`, зелёный baseline, чистое дерево.
Шаги 3–4 наследуют то же. Шаг 5 мержит ветку и создаёт деплой — старый URL
386 к этому моменту нигде, кроме истории и `.superpowers`, не упоминается.

## Источники

- real-run (shell, 2026-10-05): `git status/log`, `gh issue list` (пусто),
  `make test` (exit 0, 26+35), `make lint` (exit 0, 3 warning), grep по
  упоминаниям 386 (README + `.superpowers` — полный список).
- real-run (чтение файлов, 2026-10-05): `README.md` стр. 4, 13–21, 41–44,
  129–132 (все точки правки); `docs/agents/issue-tracker.md`
  (gh-конвенции создания issue).
- code-reading: состав issue-зеркала — из формулировки «Известных
  ограничений» README и истории task-6 (#23), номер нового issue заранее
  неизвестен (трекер 387 пуст) — вставляется по факту создания.
