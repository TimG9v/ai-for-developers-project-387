# implementation-plan

# TASK 11 — регулярные задачи по расписанию: план имплементации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for
> tracking.

- Дата: 2026-10-06 18:35 (+05)
- Ветка: шаг 1 — рабочая ветка `task-11` (создаётся от свежего `main` в
  момент старта шага); шаги 2, 3 — без коммитов кода (запуски воркфлоу
  и приёмка через GitHub)
- Шаг: план реализации (task-11)
- Spec: `docs/context/tmp/task-11/description.md` (в запросе пользователя
  файл назван `descripton.md` — фактическое имя `description.md`)

**Goal:** ночная проверка качества публичного деплоя работает без
человека: воркфлоу по `schedule` (раз в сутки, 06:00 МСК) собирает
Lighthouse-отчёт по `https://calendar-387.onrender.com`, сохраняет его
артефактом прогона, а агент читает отчёт и заводит issue с находками;
ручной запуск `workflow_dispatch` даёт тот же результат без ожидания
ночи.

**Architecture:** изменений в репо ровно одно — новый файл
`.github/workflows/opencode-scheduled.yml`. Внутри job — разделение из
спеки: сырые данные собирает обычный шаг воркфлоу (warm-up + Lighthouse
CLI → `lighthouse/report.json` → `actions/upload-artifact`), агент
(opencode action, prompt-режим) превращает JSON в выводы и создаёт
ровно одну issue. Режим аутентификации агента — OIDC/GitHub App
(как в docs Schedule Example и действующем `opencode-triage.yml`),
права — write-набор из спеки. Плюс настройка вне репо: переменная
репозитория `APP_URL` (адрес не вписан в YAML).

**Tech Stack:** GitHub Actions (`schedule` с `timezone`,
`workflow_dispatch`, `npx lighthouse@13`, `actions/upload-artifact@v7`,
`anomalyco/opencode/github@latest`), модель
`zai-coding-plan/glm-5.3-flash` + секрет `ZHIPU_API_KEY` (рабочая
связка task-8), `gh` CLI.

## Задача

`description.md` (task 11 курса): часть работы повторяется независимо
от людей — проверка скорости страниц, поиск устаревших зависимостей,
сбор технического долга; такие задачи отдаются расписанию — ночью агент
выполняет проверку, утром команда читает отчёт. Расписание — событие
`schedule` с cron из пяти полей (UTC по умолчанию; `timezone:`
Europe/Moscow можно задать рядом с cron), рядом — `workflow_dispatch`
для проверки без ожидания ночи. У задачи по расписанию нет позвавшего
человека, поэтому явно задаются две вещи: `prompt` (инструкции агенту
взять негде) и права (`contents: write` для веток и коммитов,
`pull-requests: write` и `issues: write` для PR и задач, вместе с
`id-token: write`). Результат обязан куда-то попасть: отчёт
сохраняется артефактом прогона, находки становятся задачей — в prompt
прямая инструкция «создай issue с итогами». Проверка разделена: сырые
данные собирает обычный шаг воркфлоу (Lighthouse CLI по публичной
ссылке), агент превращает отчёт в выводы и задачи; инструмент может
быть другим, важен разбор результата. Адрес приложения — переменная
репозитория, не вписанный в воркфлоу литерал. Результат: регулярная
проверка работает — воркфлоу запускается по расписанию и вручную,
отчёт сохраняется, по находкам появляются задачи.

## Ключевые решения

- **Отдельный файл `opencode-scheduled.yml`** — имя как в docs Schedule
  Example; конвенция task-8/9/10: предсуществующие воркфлоу не
  трогаются (другое событие, права, prompt).
- **Режим агента — OIDC/GitHub App, не `use_github_token: true`**:
  docs Schedule Example даёт ровно write-набор спеки + `id-token:
  write` без `GITHUB_TOKEN`; прецедент task-9 — app-токен на
  `issues: write` реально постил в issues (triage). Read-режим с
  токеном раннера (task-10) был нужен для другого сценария.
- **`cron: "0 6 * * *"` + `timezone: Europe/Moscow`** — поддержка
  `timezone` подтверждена docs workflow-syntax (real-run: «You can
  optionally specify a timezone using an IANA timezone string»);
  Москва без DST — время стабильно; 06:00 МСК — «утром команда читает
  отчёт». Один cron-элемент = не чаще раза в сутки (требование 3);
  `workflow_dispatch` — ручной и в частоту не входит.
- **`APP_URL` — переменная репозитория** (`gh variable set`): спека
  требует переменную (Settings → Secrets and variables → Actions →
  Variables); адрес публичный — переменная, не секрет; в YAML только
  `${{ vars.APP_URL }}`.
- **Warm-up шаг перед Lighthouse**: бесплатный Render засыпает после
  ~15 минут простоя, холодный старт — десятки секунд (README);
  real-run 2026-10-06: `HTTP 200` за **23.2 s** на холодном инстансе.
  Без прогрева Lighthouse измеряет холодный старт — оценки мусор.
- **Сырые данные — шаг воркфлоу, выводы — агент** (разделение из
  спеки): Lighthouse пишет `lighthouse/report.json`, экшен
  `upload-artifact` сохраняет его артефактом `lighthouse-report`,
  агент читает файл с диска раннера. `npx --yes lighthouse@13` — пин
  мажора (13.5.0 — latest, real-run npm); один json-файл, без
  Lighthouse CI (lhci тянет свой server/статику — избыточен, спека
  допускает любой инструмент).
- **Дата прогона в заголовке issue — через step output** (`date +%F`
  → `steps.today.outputs.date` → подстановка в prompt): агенту нельзя
  доверять знание текущей даты; дата в заголовке защищает от дублей
  при повторных ручных запусках и различает ночные прогоны.
- **Побочный эффект — авто-триаж новой issue**: агент-созданная issue
  триггерит `opencode-triage.yml` (`issues: opened`) — на ней появится
  триаж-комментарий. Это существующий конвейер, не дефект;
  `share: false` в prompt-режиме не даёт самопробуждения
  `opencode.yml` (task-9, real-run).
- **Через PR в `main`, мерж `--merge`** — конвенция task-7–10; коммит
  воркфлоу — `ci:` (Conventional Commits). `workflow_dispatch` работает
  только для файла на дефолтной ветке (docs, real-run) — ручной запуск
  строго после мержа шага 1.
- **Отчёт — артефакт прогона (не «или в issue»)**: спека даёт выбор,
  артефакт надёжнее — сырой JSON полнее пересказа агента; issue
  дублирует выводы в человекочитаемом виде.

## Механика

- Шаг 1 — настройка репо вне git: `APP_URL` (не коммит); ветка
  `task-11`, файл `opencode-scheduled.yml`, PR → `main`, мерж,
  регистрация воркфлоу `active`. Шаг 2 — ручной запуск
  `workflow_dispatch`, зелёный прогон: warm-up → Lighthouse → артефакт
  → агент → issue с находками (плюс ожидаемый триаж-комментарий).
  Шаг 3 — приёмка всех пяти требований по описанию; ночной
  schedule-прогон — опциональное наблюдение на следующее утро.
- Линейность 1 → 2 → 3: шаг 2 требует файл в `main` (dispatch только
  с дефолтной ветки), шаг 3 требует артефакты и issue из шага 2.
- Расход: 1 модельный прогон на каждый запуск воркфлоу (шаг 2 — один;
  далее ≤1/сутки по расписанию) + 1 побочный триаж-прогон на
  созданную issue.
- `docs/context/tmp/task-11/` — рабочий каталог; переезд в
  `docs/context/task-11/` — решение пользователя после приёмки
  (образец task-8/9/10), в объём шагов не входит.
- Секреты: значение `ZHIPU_API_KEY` не проходит через чат, логи и
  файлы репозитория (no-secrets-exposure); в отчётах — только маски.
  `APP_URL` — не секрет, но и её значение в план вынесено один раз
  (README/спека), в YAML не вписывается.
- Cycle-report — отдельный контекст-файл после приёмки (образец
  task-8/9/10); шаг 3 готовит для него материал (номера прогонов,
  имя артефакта, номер issue, хронология).

## Текущее состояние (real-run 2026-10-06 18:35 +05)

| Что                | Состояние                                                                                                       |
| ------------------ | --------------------------------------------------------------------------------------------------------------- |
| Ветка / дерево     | `main` @ `c646bb2` «Merge pull request #14 …», дерево чистое                                                    |
| CI на `main`       | Backend CI, Frontend CI, hexlet-check, Release Please — success (прогоны 37469573xxx, 2026-10-06 13:17 UTC)     |
| Воркфлоу           | 8 active: backend-ci, frontend-ci, hexlet-check, opencode, opencode-triage, opencode-review, Release Please, Security; `opencode-scheduled` нет |
| Переменные репо    | пусто — `gh variable list` не возвращает ни одной (спека требует `APP_URL`)                                     |
| Секреты            | `HEXLET_ID` (2026-10-01), `ZHIPU_API_KEY` (2026-10-06) — `gh secret list`                                       |
| Issues             | #9, #5, #1 OPEN; #7 CLOSED (merge #14); открытых issues от агента нет                                           |
| Публичный деплой   | `https://calendar-387.onrender.com` (README); real-run: `HTTP 200` за 23.2 s на холодном инстансе               |
| Lighthouse         | latest **13.5.0** (`npm view lighthouse version`, real-run)                                                     |
| upload-artifact    | latest **v7.0.1** (`gh api repos/actions/upload-artifact/releases/latest`, real-run)                            |
| opencode-конвенция | OIDC-режим (App) в `opencode.yml`/`opencode-triage.yml`; write на issues подтверждён task-9                     |

## Требования → шаги (description.md)

| # | Требование                                                                              | Шаг |
| - | --------------------------------------------------------------------------------------- | --- |
| 1 | Воркфлоу с `schedule` (cron задан) и `workflow_dispatch` в `.github/workflows/`         | 1   |
| 2 | `prompt` задан; права `contents`/`pull-requests`/`issues`/`id-token` — `write`          | 1   |
| 3 | Частота расписания — не чаще одного запуска в сутки                                     | 1, 3 |
| 4 | Хотя бы один успешный прогон; отчёт сохранён артефактом (или опубликован в задаче)      | 2   |
| 5 | По находкам отчёта заведён минимум один issue                                           | 2, 3 |

Задачи описания → шаги: (1) воркфлоу с `schedule` + `workflow_dispatch`,
cron с учётом UTC → 1; (2) права и prompt → 1; (3) сбор данных
(Lighthouse CLI), адрес — переменная репозитория → 1; (4) ручной
запуск, успешный прогон → 2; (5) отчёт артефактом, найти во вкладке
Actions → 2; (6) issue по находке → 2 (проверка — 3).

## Шаги

Все файлы шагов — в `docs/context/tmp/task-11/`, имена с префиксом
`20261006183526-` (опущен в таблице).

| Шаг | Файл                                    | Суть                                                                                  | Артефакт                                          |
| --- | --------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------- |
| 1   | `step-1-variable-and-workflow-to-main.md` | переменная `APP_URL`; `opencode-scheduled.yml` (schedule + dispatch + Lighthouse + агент); PR → `main` | воркфлоу active в `main`, переменная задана       |
| 2   | `step-2-manual-run-artifact-issue.md`   | ручной запуск → success: warm-up, Lighthouse, артефакт, issue от агента                | прогон success, артефакт `lighthouse-report`, issue |
| 3   | `step-3-acceptance.md`                  | приёмка пяти требований; опционально — ночной schedule-прогон                          | чек-лист приёмки, материал для cycle-report       |

## Global Constraints

- Новый файл строго `.github/workflows/opencode-scheduled.yml`;
  события: `schedule` с одним cron-элементом `0 6 * * *` и
  `timezone: Europe/Moscow`, плюс `workflow_dispatch` (спека +
  docs Schedule Example).
- Права job: `id-token: write`, `contents: write`,
  `pull-requests: write`, `issues: write` — ровно спека и docs
  Schedule Example; write-набор обязателен, потому что schedule-прогон
  идёт без пользовательского контекста permission-check (docs).
- Режим агента — OIDC/GitHub App: env `ZHIPU_API_KEY`, модель
  `zai-coding-plan/glm-5.3-flash`, без `use_github_token` и без
  `GITHUB_TOKEN` в env (рабочая связка task-8, docs Schedule Example).
- `share: false`; prompt непустой, с прямой инструкцией «создай
  issue с итогами» и запретом веток/коммитов/PR (спека).
- Адрес приложения — только через `${{ vars.APP_URL }}`; литерал
  адреса в YAML не вписывать (спека).
- Артефакт: `actions/upload-artifact@v7`, путь
  `lighthouse/report.json`, имя `lighthouse-report`.
- Предсуществующие воркфлоу (`opencode.yml`, `opencode-triage.yml`,
  `opencode-review.yml`, `release-please.yml`, CI, `security.yml`,
  `hexlet-check.yml`) не изменяются; `hexlet-check.yml` не трогать.
- Коммиты — Conventional Commits: воркфлоу — `ci:`, контекст-файлы —
  `docs:` (AGENTS.md); изменения в `main` — только через PR, мерж
  `--merge`.
- Значения секретов не проходят через чат, логи и файлы — только маски
  (no-secrets-exposure).
- Пин инструментов: `lighthouse@13` (13.5.0), `upload-artifact@v7`
  (7.0.1), `checkout@v6` (консистентно с opencode-файлами репо) —
  real-run на момент плана.

## Review Focus (что скорее всего укусит)

1. **`workflow_dispatch` не сработает до мержа файла в `main`** —
   docs: «This trigger only receives events when the workflow file is
   on the default branch». Ожидание: ручной запуск только после мержа
   шага 1; попытка раньше — воркфлоу не виден во вкладке Actions.
   Пин — шаг 2 (предусловие).
2. **Cold start Render портит метрики** — без прогрева Lighthouse
   измеряет десятки секунд ожидания (real-run: 200 за 23.2 s). Ожидание:
   в логе прогона warm-up-шаг с retry до `200` до запуска Lighthouse.
   Пин — шаг 2 (лог warm-up).
3. **Агент не создал issue при success-прогоне** — в schedule/prompt-
   режиме вывод по умолчанию идёт в логи и PR; создание issue — не
   дефолт. Ожидание: прямая инструкция в prompt; если issue нет —
   усилить prompt, докоммит `ci:` новым PR. Пин — шаг 2.
4. **`APP_URL` не задана** — `curl`/`npx lighthouse` падают на пустом
   аргументе с невнятной ошибкой. Ожидание: переменная существует до
   первого прогона (`gh variable list`). Пин — шаг 1 (создание),
   шаг 2 (предусловие).
5. **Дубли issues при повторных ручных запусках** — каждый прогон
   создаёт свою issue. Ожидание: дата в заголовке делает заголовок
   уникальным в пределах суток; повторный запуск того же дня — осознанный
   источник второй issue (принято; дедупликация — открытый вопрос).
   Пин — шаг 2 (отрицательная проверка).
6. **Авто-триаж новой issue** — `opencode-triage.yml` сработает на
   `issues: opened` и оставит комментарий. Ожидание: это существующий
   конвейер, не дефект; зацикливания нет (комментарий триажа без
   `/oc`, `share: false`). Пин — наблюдение в шаге 2.
7. **Chrome на раннере** — ubuntu-latest включает Chrome stable;
   если Lighthouse не найдёт браузер, шаг упадёт с внятной ошибкой;
   fallback — `export CHROME_PATH=/usr/bin/google-chrome-stable`
   докоммитом. Пин — шаг 2 (лог прогона).
8. **Ночной schedule-прогон не придёт мгновенно** — первый по
   расписанию в 06:00 МСК следующего дня; приёмка на нём не держится
   (требование 4 закрывает `workflow_dispatch`). Пин — шаг 3
   (опционально).

## Открытые вопросы

1. **Время cron** — дефолт `0 6 * * *` Europe/Moscow («утром команда
   читает отчёт»); другое время — решение пользователя, меняется
   одной строкой.
2. **HTML-версия отчёта** — второй запуск Lighthouse с `--output html`
   (человекочитаемый артефакт); в минимальную спеку не входит.
3. **Политика ежедневных issues** (авто-закрытие старых, label,
   дедупликация повторных ручных прогонов) — за пределами минимальной
   спеки; решается после первой недели ночных прогонов.
4. **Финальное место контекстов** — переезд `docs/context/tmp/task-11/`
   → `docs/context/task-11/` после приёмки (образец task-8/9/10),
   отдельным `docs:`-коммитом, в шаги не входит.

## Критерии приёмки (description.md, «Требования» + «Результат шага»)

- В `.github/workflows/` есть воркфлоу с событиями `schedule`
  (cron `0 6 * * *`, `timezone: Europe/Moscow`) и
  `workflow_dispatch`; воркфлоу зарегистрирован `active` (требование 1).
- У воркфлоу задан непустой `prompt`; права `id-token: write`,
  `contents: write`, `pull-requests: write`, `issues: write`
  (требование 2).
- Cron-элемент один — запуск не чаще раза в сутки (требование 3).
- Существует успешный прогон (`workflow_dispatch`); во вкладке
  Actions → прогон → Artifacts лежит `lighthouse-report` с JSON-отчётом
  (требование 4).
- Существует минимум одна issue с находками: заголовок с датой прогона,
  тело содержит оценки категорий Lighthouse и рекомендации (требование 5).
- Результат шага: регулярная проверка работает — запуск по расписанию
  и вручную, отчёт сохранён, по находкам заведена задача; адрес
  приложения взят из переменной репозитория.

## Источники выводов

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — Schedule
  Example (`opencode-scheduled.yml`: `schedule` + cron, permissions
  `id-token`/`contents`/`pull-requests`/`issues` write, prompt обязателен;
  «Scheduled workflows run without a user context to permission-check,
  so the workflow must grant contents: write and pull-requests: write»),
  Supported Events (`schedule`/`workflow_dispatch`: «Output goes to
  logs and PRs», prompt required), Configuration (`model`, `prompt`,
  `share` default true для публичных репо); docs.github.com workflow
  syntax — `on.schedule` (UTC по умолчанию; `timezone` — IANA-строка
  рядом с `cron`; пример; минимальный интервал 5 минут; расписание
  работает с последним коммитом дефолтной ветки), `on.workflow_dispatch`
  («This trigger only receives events when the workflow file is on the
  default branch»).
- real-run (shell, 2026-10-06): `git branch --show-current` /
  `git log --oneline -5` / `git status --porcelain` (main @ `c646bb2`,
  чисто); `ls .github/workflows/` (8 yml + README, scheduled нет);
  `gh workflow list` (8 active); `gh secret list` (HEXLET_ID,
  ZHIPU_API_KEY); `gh variable list` (пусто); `gh issue list --state
  all` (#9/#5/#1 OPEN, #7 CLOSED); `gh run list` (CI success на main);
  `npm view lighthouse version` (13.5.0); `gh api repos/actions/
  upload-artifact/releases/latest` (v7.0.1); `curl -s -o /dev/null -w`
  `https://calendar-387.onrender.com` (HTTP 200, 23.2 s — cold start);
  `gh repo view` (TimG9v/ai-for-developers-project-387).
- real-run (чтение файлов, 2026-10-06): `description.md` (требования,
  задачи, результат); `.github/workflows/opencode.yml` (связка
  env/model, стиль шагов), `opencode-triage.yml` (прецедент app-токена
  с `issues: write`, prompt на русском, `share: false`),
  `opencode-review.yml` (task-10), `hexlet-check.yml` (DO NOT EDIT);
  `README.md` (публичный деплой на Render, cold start, in-memory
  хранилище).
- code-reading: warm-up-шаг — следствие cold-start из README и замера
  23.2 s; дата в заголовке issue через step output — детерминизм
  (знание текущей даты агентом не гарантируется); «ровно одна issue
  за прогон» — поведенческая инструкция prompt; дубликаты при повторных
  dispatch того же дня — следствие, принято осознанно; зацикливания
  триажа нет — комментарий триажа без `/oc` не будит `opencode.yml`
  (подтверждено task-9).
