# step-5-render-deploy-acceptance

- Дата: 2026-10-05 10:25
- Ветка: task-7 → main (мерж в этом шаге)
- Шаг: план шага 5/5 — деплой на Render, README-ссылка, приёмка цикла

Исходный план: `20261005102500-implementation-plan.md`.

## Цель

Влить `task-7` в `main`, развернуть приложение на Render из репозитория
387 через MCP (новый сервис — существующие трекают только 386),
верифицировать публичный URL сквозным сценарием, вписать ссылку в README
и провести приёмку всего цикла: зелёный CI, hexlet-check, переезд
контекстов в `docs/context/task-7/`.

## Interfaces

- Consumes: ветка `task-7` с шагами 1–4 (README без старого URL, контракт
  no-diff, сценарий пройден локально и в контейнере); Render MCP:
  workspace `hx-study` (`tea-dauujue0tbcc73cq2200`), инструменты
  `create_web_service`, `list_deploys`, `get_deploy`, `trigger_deploy`,
  `list_logs`; Dockerfile из шага 4 (собран локально — репетиция).
- Produces: публичный URL 387 (в README «Публичный деплой»); сервис
  Render `<name>.onrender.com` с autoDeploy из `main`; контексты цикла в
  `docs/context/task-7/`.

## Предусловия (real-run перед стартом)

- [ ] 1. `git pull && git status --porcelain` — пусто; ветка `task-7`;
  шаги 1–4 выполнены, их «Проверки» заполнены.
- [ ] 2. Render MCP отвечает:

  ```txt
  инструменты: list_workspaces → workspace hx-study (tea-dauujue0tbcc73cq2200)
  ```

  Ожидание: workspace виден. Если MCP недоступен (сессия без
  интеграции) — стоп: деплой выполняет пользователь в дашборде по
  параметрам из действия 3, агент верифицирует URL и делает README
  (fallback задачи 6, адаптированный).

## Действия

### A. Мерж

- [ ] 1. Запушить ветку и открыть PR:

  ```bash
  git push -u origin task-7
  gh pr create --base main --head task-7 \
    --title "task-7: full project cycle on the inherited codebase" \
    --body "Адаптация копии 386 под цикл проекта 387: гигиена ссылок,
  верификация контракта/приложения/образа, деплой.
  Планы: docs/context/tmp/task-7-pre/20261005102500-implementation-plan.md"
  ```

- [ ] 2. Дождаться зелёного CI на PR и влить:

  ```bash
  gh pr checks --watch
  gh pr merge --merge
  git checkout main && git pull
  ```

  Ожидание: все проверки зелёные (backend-ci, frontend-ci, security,
  hexlet-check, release-please), merge-коммит в `main`. Красный security
  на унаследованных advisory — по пину Review Focus №6 общего плана:
  разбор отдельным контекстом, мерж отложить.

### B. Деплой

- [ ] 3. Создать сервис (MCP `create_web_service`):

  ```txt
  create_web_service({
    workspaceId: "tea-dauujue0tbcc73cq2200",
    name: "calendar-387",            // → calendar-387.onrender.com
    repo: "https://github.com/TimG9v/ai-for-developers-project-387",
    branch: "main",
    runtime: "docker",               // Dockerfile, dockerContext "."
    plan: "free",
    region: "frankfurt",
    autoDeploy: "yes"
  })
  ```

  Имя `calendar-387` свободно (real-run list_services 2026-10-05:
  заняты `calendar-zvonok` и `ai-for-developers-project-386`).
  `BACKEND_PORT` не задаётся — дефолт 8081 совпадает с константой прокси;
  `PORT` платформа передаёт сама.

- [ ] 4. Дождаться первого деплоя (free docker-сборка 10–20 минут):
  `list_deploys`/`get_deploy` до статуса `live`; при ошибке сборки —
  `list_logs` и разбор (первый кандидат — отличие окружения сборки от
  локальной репетиции шага 4).

- [ ] 5. Верификация публичного URL (холодный старт free-плана: первый
  запрос может отвечать десятки секунд — прогревочный curl не считать
  отказом):

  ```bash
  BASE=https://calendar-387.onrender.com
  curl -s -o /dev/null -w '%{http_code}\n' "$BASE/health"     # 200 (после прогрева)
  for p in / /booking /admin /health; do
    printf '%s %s\n' "$p" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE$p")"
  done
  ```

  Затем — HTTP-сценарий шага 3 (действие 3) на `$BASE` и браузерный
  сценарий шага 3 (часть B) на публичном URL, одним заходом
  (in-memory-данные переживут сессию, но не ре-деплой).

### C. README и приёмка

- [ ] 6. Вписать URL в секцию «Публичный деплой» (вместо формулировки
  «ссылка появится», шаг 1):

  ```markdown
  ## Публичный деплой

  **https://calendar-387.onrender.com** — Render, бесплатный план. Две
  особенности бесплатного инстанса:

  - после ~15 минут простоя первый запрос отвечает с задержкой на холодный
    старт (десятки секунд);
  - данные хранятся в памяти процесса — ре-деплой или рестарт контейнера
    обнуляет типы встреч, слоты и записи.
  ```

  Скриншоты публичного URL (страница записи, админка со сценарием B)
  снять, положить в `docs/context/task-7/` и сослаться в README по
  образцу текущих (стр. 23–29) — `docs:`-коммит. Проверить глазами, что
  унаследованные скриншоты `docs/context/task-6/*.png` рендерятся
  (пин Review Focus №7).

- [ ] 7. Финальная приёмка:

  ```bash
  git status --porcelain          # пусто (после docs-коммита)
  gh run list --limit 5           # все workflows success на main
  ```

  Сверка критериев приёмки общего плана: все пункты закрыты шагами 1–5.

- [ ] 8. Переезд контекстов (по образцу task-6, `docs:`-коммит):

  ```bash
  mkdir -p docs/context/task-7
  git mv docs/context/tmp/task-7-pre/*.md docs/context/task-7/
  git mv docs/context/tmp/task-7-pre/description.md docs/context/task-7/
  git commit -m "docs: move task-7 planning contexts to committed docs"
  git push
  ```

  Если этапные описания 387 появятся до этого момента — переехать вместе
  с планами; папка `tmp/task-7-pre` опустеет и исчезнет из git.

## Файлы

- Modify: `README.md` (секция «Публичный деплой», скриншоты).
- Create: `docs/context/task-7/*.md` (переезд из `tmp/task-7-pre`),
  скриншоты публичного деплоя.
- Не трогать: сервисы Render 386 (`calendar-zvonok` и rust-сервис) —
  история проекта 386, не перенацеливать и не удалять.

## Проверка (real-run)

| # | Действие / команда                                    | Ожидаемый результат                                          |
| - | ----------------------------------------------------- | -------------------------------------------------------------- |
| 1 | `gh pr checks` → `gh pr merge --merge`                | все checks зелёные, PR влит, `main` обновлён                   |
| 2 | `create_web_service` (параметры действия 3)           | сервис создан, url `https://calendar-387.onrender.com`         |
| 3 | `get_deploy`                                          | статус `live`                                                  |
| 4 | for-цикл по `$BASE` (4 пути)                          | четыре `200`                                                   |
| 5 | HTTP-сценарий шага 3 на `$BASE`                       | тип+слот+бронь; повтор `409`; upcoming содержит гостя          |
| 6 | браузерный сценарий шага 3 (часть B) на `$BASE`       | 4 пункта пройдены, скриншоты сняты                             |
| 7 | `gh run list --limit 5` на main                       | backend, frontend, security, hexlet-check, release-please — success |
| 8 | `git status --porcelain`, `ls docs/context/task-7/`   | чисто; планы цикла лежат в `docs/context/task-7/`              |

## Коммит

Два docs-коммита агентом: `docs: add deployed app link and screenshots to
readme` (действие 6) и `docs: move task-7 planning contexts to committed
docs` (действие 8). Оба — в `main` после мержа PR (обоснование —
«Открытый вопрос» №5 общего плана: URL известен только после деплоя,
сервис создаётся из `main`).

## Приёмка цикла (сводная)

- Контракт: `make generate` no-diff (шаг 2).
- Интерфейс/бэкенд: сценарий локально и в браузере (шаг 3).
- Тесты: 26 backend + 35 frontend зелёные (шаги 1, 3).
- Docker: образ собирается, автостарт, `$PORT` (шаг 4).
- Облако: публичный URL живой, сценарий на нём пройден (этот шаг).
- Гигиена: README без упоминаний 386 вне истории (шаги 1, 5).

## Источники

- real-run (Render MCP, 2026-10-05): `list_workspaces` (hx-study,
  tea-dauujue0tbcc73cq2200), `list_services` (занятые имена; оба сервиса —
  386; free plan, frankfurt/oregon), сигнатура `create_web_service`
  (runtime docker: Dockerfile по умолчанию, autoDeploy, branch).
- real-run (shell, 2026-10-05): `gh run list` (пять workflows на push
  `9bdd91f`, security in_progress), `gh issue/pr list` (пусто — PR будет
  первым), README стр. 13–29 (формат секции деплоя и скриншотов).
- real-run (история 386, чтение, 2026-10-05): task-6 step-4/step-5
  (практика деплоя и приёмки: прогрев, in-memory, переезд контекстов,
  `docs:`-коммиты), task-5 step-7 (браузерная приёмка).
- code-reading: тайминг free docker-сборки (10–20 мин) — из опыта task-6;
  фактическое значение фиксируется в «Проверке».
