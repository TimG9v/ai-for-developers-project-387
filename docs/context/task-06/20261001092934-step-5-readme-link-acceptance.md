# step-5-readme-link-acceptance

- Дата: 2026-10-01 09:29
- Ветка: task-6 (рабочая)
- Шаг: план шага 5/5 — README со ссылкой, приёмка: CI, PR, закрытие #20, переезд контекстов

Исходный план: `20261001092929-implementation-plan.md`.

## Цель

Публичная ссылка из шага 4 добавлена в репозиторий (README), README описывает
docker-запуск; ветка `task-6` уходит PR'ом в `main`, GitHub Actions зелёный
(включая hexlet-check — автопроверку, которая по descripton собирает образ),
PR влит, сервис хостинга переключён на `main`, #20 закрыт. Контексты шага
переезжают в `docs/context/task-6/`.

## Предусловия

1. Шаги 1–4 выполнены; `<URL>` из шага 4 живой (`curl -sf <URL>/health`).
2. Все коммиты шагов запушены в `task-6` (`git status --porcelain` — пусто).

## Действия

1. **README** — заполнить секции «Установка» и «Использование» (сейчас там
   комментарии-плейсхолдеры). Целевой вид (текст подставить дословно, `<URL>`
   — из шага 4):

   ````markdown
   ## Установка

   ```bash
   git clone https://github.com/TimG9v/ai-for-developers-project-386.git
   cd ai-for-developers-project-386
   ```

   Локальный запуск (Rust + Node по `.tool-versions`/`.nvmrc`):

   ```bash
   make dev   # backend :8081, frontend :3000
   ```

   ## Использование

   Опубликованное приложение: <URL>

   Тот же образ локально:

   ```bash
   docker build -t calendar .
   docker run -e PORT=8080 -p 8080:8080 calendar
   # http://127.0.0.1:8080 — страница записи, /admin — админка, /health — статус
   ```

   Контейнер сам поднимает backend и frontend; приложение отвечает на порту
   из переменной `PORT`.
   ````

2. Полный прогон: `make test && make lint` — exit 0 (3 предсуществующих
   eslint-предупреждения не блокируют).
3. Пуш и PR:

   ```bash
   git push -u origin task-6
   gh pr create --base main --title "feat: docker deploy (task-6)" \
     --body "Dockerfile (один контейнер, PORT), same-origin API-прокси (#20), публичная ссылка в README"
   ```

4. Дождаться зелёных проверок PR: Backend CI, Frontend CI, Security, и —
   главное для этого шага — **hexlet-check** (автопроверка Хекслета собирает
   образ по Dockerfile и запускает приложение — descripton, требование 1).
   Красный hexlet-check = образ в репо не проходит проверку: разбирать до
   merge.
5. **Merge** (после ревью пользователя — по практике task-5, PR #21).
6. После merge:
   - сервис хостинга переключить на ветку `main` (MCP-вызов или дашборд),
     дождаться ре-деплоя, `curl -sf <URL>/health` — 200;
   - проверить закрытие #20: `gh issue view 20 --jq .state` — `closed`
     («Closes #20» в коммите шага 2 сработал на merge). Если не закрылся —
     `gh issue close 20 --comment "закрыто same-origin прокси, шаг 2 task-6"`.
7. **Переезд контекстов** (по образцу task-5: `docs: move contexts ...`):

   ```bash
   mkdir -p docs/context/task-6
   git mv docs/context/tmp/task-6/*.md docs/context/task-6/
   git commit -m "docs: add task-6 contexts (docker deploy)"
   git push
   ```

   (Ветки `tmp`-копии `descripton.md` переезжают вместе с контекстами — так
   же сделано в task-5.)

8. Финальная проверка ссылки после переключения на `main` (real-run):
   `/health`, `/`, `/booking` — 200.

## Файлы

- Modify: `README.md` (установка, использование, публичная ссылка).
- Move: `docs/context/tmp/task-6/*.md` → `docs/context/task-6/`.
- Не трогать: `Dockerfile`, код приложений, `hexlet-check.yml`.

## Проверка (real-run)

| # | Команда                                                        | Ожидаемый результат                          |
| - | -------------------------------------------------------------- | -------------------------------------------- |
| 1 | `make test && make lint`                                       | exit 0                                       |
| 2 | `gh pr checks <номер>`                                          | все зелёные, включая hexlet-check            |
| 3 | `gh pr view <номер> --json state --jq .state`                   | `MERGED`                                     |
| 4 | `gh issue view 20 --jq .state`                                  | `closed`                                     |
| 5 | `curl -sf <URL>/health` после переключения на main              | 200 `{"status":"ok"}`                        |
| 6 | `ls docs/context/task-6/`                                       | планы шага на месте, tmp-папка пуста         |
| 7 | `git status --porcelain`                                        | пусто                                        |

## Критерии приёмки шага (descripton.md, «Результат шага»)

- Dockerfile в репо; hexlet-check (сборка образа + запуск) зелёный.
- Контейнер стартует приложение автоматически (проверено в шаге 3, закреплено
  hexlet-check).
- Приложение работает по `PORT` (шаги 1–3) и по публичной ссылке (шаг 4).
- Публичная ссылка в репозитории (README, этот шаг).

## Коммит

Агент в ветке `task-6`:

- `docs: add deployed app link and docker usage to README`;
- после merge — `docs: add task-6 contexts (docker deploy)` в `main`
  (по практике task-5).

## Источники

- real-run (чтение файлов, 2026-10-01): `README.md` (секции-плейсхолдеры
  «Установка»/«Использование», бейдж hexlet-check), `Makefile` (команды
  dev/test/lint), `docs/agents/issue-tracker.md` (закрытие issue),
  `.github/workflows/hexlet-check.yml` (не трогать; автопроверка на каждый
  пуш).
- real-run (чтение, 2026-10-01): `docs/context/tmp/task-5/20260930173909-implementation-plan.md`
  (паттерн приёмки: PR, зелёный CI, переезд контекстов), история коммитов
  task-5 (`git log`: `docs: move contexts...`, PR #21).
- code-reading: формулировки README — из Dockerfile/entrypoint шага 3 и
  требований descripton.md; порядок «merge → переключение сервиса на main» —
  из развилки ветки деплоя (общий план, открытый вопрос 2).
