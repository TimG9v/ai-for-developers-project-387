# implementation-plan

# TASK 12 — гигиена агентного процесса: план имплементации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for
> tracking.

- Дата: 2026-10-08 21:31 (+05)
- Ветка: шаг 1 и 2 — рабочая ветка `task-12` (существует локально,
  база `4689783` отстаёт от `main` — перед правками fast-forward на
  свежий `main` командой `git merge --ff-only origin/main`;
  уникальных коммитов на ветке нет, fast-forward безопасен);
  шаг 3 — без коммитов кода (комментарии-тесты и приёмка через GitHub)
- Шаг: план реализации (task-12)
- Spec: `docs/context/tmp/task-12/description.md` (в запросе пользователя
  файл назван `descripton.md` — фактическое имя `description.md`)

**Goal:** агентный процесс в GitHub предсказуем: воркфлоу агента не
запускаются на события от ботов и посторонних, команда вызова одна
(`/oc` через `mentions`), права минимальны (в `opencode-review.yml`
`issues: write` понижен до `read`), сессии не публикуются (явный
`share: false`), а устройство воркфлоу, решения по команде, шаре и
кругу вызывающих плюс самооценка работы с агентом записаны в README.md.

**Architecture:** новых файлов и новых событий нет — правки только в
трёх существующих агентных воркфлоу (усиление `if`-условий, `mentions`,
`share`, понижение одного write-права) и в README.md (новый раздел
«Агентные воркфлоу»: таблица девяти воркфлоу проекта, принятые решения,
самооценка). `opencode-scheduled.yml` по правам не меняется: его
write-набор оправдан docs Schedule Example и живым прецедентом
сайд-канального PR #27. Два PR: `ci:` (воркфлоу) и `docs:` (README).

**Tech Stack:** GitHub Actions (`if`-условия по `user.type` и
`author_association`, `permissions`, параметры экшена
`anomalyco/opencode/github@latest`: `mentions`, `share`), README.md,
`gh` CLI, `python3 + pyyaml` для локальной валидации.

## Задача

`description.md` (task 12 курса): новых сценариев нет — задача
закрепить предсказуемость агентного процесса. Пять мест поломок:
(1) триггеры — воркфлоу на комментарий запускается и на ответ самого
агента, без фильтра по автору получается петля и сгорание минут и
токенов; лечится отсечением ботов и узкой командой вызова (параметр
`mentions`, по умолчанию широкий `/opencode,/oc`); (2) права — запись
нужна только там, где агент создаёт коммиты, задачи и PR, раздача по
воркфлоу отдельно, а не дефолтом на весь репозиторий; (3) расходы —
модель под задачу (параметры `model`, `variant`, `agent`), сводка
в таблицу в README; (4) публикация сессий — для открытых репозиториев
`share` включён по умолчанию, решение принимается осознанно;
(5) доступ к вызову — позвать агента может любой комментатор, круг
ограничивается условием в воркфлоу. Требования: бот-условия во всех
агентных воркфлоу; набор команд задан явно через `mentions` либо
в README записано почему дефолт; права по воркфлоу с записью только
по необходимости; README-таблица воркфлоу (событие, модель, назначение,
где смотреть прогоны); решение по `share` принято и записано;
короткая самооценка (что агент закрыл с первого прохода, где были
итерации). Результат: петель нет, права минимальные, расходы под
контролем, решения записаны в репозитории.

## Ключевые решения

- **Бот-фильтр в `opencode-triage.yml`** (`if: github.event.issue.user.type
  != 'Bot'`) — прямой фикс доказанного инцидента: Lighthouse-прогон
  создал issue #26 от `app/opencode-agent`, автотриаж стартовал и упал
  на внутреннем perm-гейте экшена («opencode-agent[bot] does not have
  write permissions», прогон 37757599684, failure + мусорный комментарий
  от бота). Прогон не должен стартовать вовсе: у бот-автора нет
  collaborator-прав, агент в любом случае не выполнится.
- **`mentions: /oc` в `opencode.yml`** — сужение дефолтной пары
  `/opencode,/oc` до одной команды: вся реальная практика task-8–11 —
  `/oc`; подстрока `/opencode` встречается в URL (`opencode.ai/...`) и
  была источником самопробуждения на комментариях бота (task-8);
  `if`-гейт упрощается до одной подстроки `'/oc'`, которая в ссылках
  opencode.ai не встречается. Альтернатива спеки (оставить дефолт и
  описать в README) отброшена: спека называет дефолт «широким» и прямо
  рекомендует узкую команду.
- **Круг вызывающих — условие `author_association`** в `if`
  `opencode.yml` (`OWNER`/`COLLABORATOR`/`MEMBER`): сейчас чужой
  `/oc`-комментарий в публичном репозитории стартует прогон, который
  падает на внутреннем perm-гейте с мусорным комментарием (прецедент
  task-8); с условием посторонний комментарий даёт `skipped` без
  старта агента — нулевой расход. Внутренний гейт экшена остаётся
  вторым слоем.
- **Явный `share: false` в `opencode.yml`** — сейчас параметр
  закомментирован (`# share: true`), действует дефолт «true для
  публичных репозиториев»: сессии агента публикуются по ссылке вместе
  с контекстом. Решение закрывается явно: false — единообразно с
  триажем/ревью/расписанием, контекст репозитория не раскрывается,
  заодно исчезает вектор «ссылка с `/opencode` в комментарии бота».
- **`opencode-review.yml`: `issues: write` → `issues: read`** — ревью
  пишет только комментарий к PR (`pull-requests: write`), запись в
  issues не нужна. При явном `permissions`-блоке GitHub все незаданные
  скоупы становятся `none`, поэтому `read` задаётся явно — чтение
  связанных issue сохраняется. Соответствие docs Pull Request Example
  (read-набор для ревью).
- **`opencode-scheduled.yml` по правам не меняется**: docs Schedule
  Example прямо требует write-набор (прогон идёт без user context для
  permission-check), `issues: write` нужен для issue по находкам,
  а сайд-канальный PR #27 доказал, что output-канал экшена реально
  пользуется `contents: write`/`pull-requests: write`. Решение
  документируется в README вместо изменения.
- **`opencode.yml` по правам не меняется**: только `id-token: write` —
  точное соответствие docs Manual Setup; агент пишет в репозиторий
  через App-токен, не через токен воркфлоу.
- **README — таблица всех девяти воркфлоу** (пять CI без модели, четыре
  агентных с моделью): событие, модель, назначение, где смотреть
  прогоны; рядом — принятые решения (команда `/oc`, круг вызывающих,
  бот-фильтры, `share: false`, расходы: timeouts, concurrency,
  дешёвая модель) и самооценка по материалам cycle-reports task-8–11.
- **Два отдельных PR** — `ci:` (3 файла воркфлоу) и `docs:` (README):
  раздельное ревью и откат; Conventional Commits (AGENTS.md).

## Механика

- Шаг 1 — ветка `task-12` fast-forward на `origin/main`, правки трёх
  файлов воркфлоу, локальная валидация YAML + grep-проверки, PR `ci:`,
  мерж. Шаг 2 — README-раздел (таблица + решения + самооценка),
  PR `docs:`, мерж. Шаг 3 — приёмка семи требований: grep по `main`,
  живые комментарии-тесты (`/oc` → success без ссылки на сессию;
  `/opencode` → skipped; комментарий без команды → skipped),
  опциональный dispatch `opencode-scheduled` для живой проверки
  бот-фильтра триажа.
- Линейность 1 → 2 → 3: шаг 2 описывает в README то, что уже в коде
  после шага 1; шаг 3 проверяет итог в `main`.
- Расход: один модельный прогон на шаг 3 (комментарий `/oc ping`) +
  опционально один dispatch `opencode-scheduled`; skipped-прогоны
  бесплатны.
- `docs/context/tmp/task-12/` — рабочий каталог; переезд в
  `docs/context/task-12/` — решение пользователя после приёмки
  (образец task-8–11), в объём шагов не входит.
- Секреты: значения `ZHIPU_API_KEY`, `RENDER_API_KEY` не проходят
  через чат, логи и файлы (no-secrets-exposure); в отчётах только
  маски.

## Текущее состояние (real-run 2026-10-08 21:31 +05)

| Что                      | Состояние                                                                                                                                   |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Репозиторий              | `TimG9v/ai-for-developers-project-387`, **PUBLIC** (`gh api` `.private=false`), default branch `main`                                       |
| `main`                   | `d54aae7` «Merge pull request #36 …», CI зелёный                                                                                            |
| Локальный чекаут         | ветка `task-12` @ `4689783` — отстаёт от `main` (без PR #28–36); уникальных коммитов нет                                                    |
| Активные воркфлоу        | 9 (`gh workflow list`): Backend CI, Frontend CI, hexlet-check, Release Please, Security + 4 opencode-*                                      |
| `opencode.yml`           | if бот-фильтр есть; `mentions` не задан (дефолт `/opencode,/oc`); `share` закомментирован → **сессии публикуются**; права `id-token: write` |
| `opencode-triage.yml`    | **бот-фильтра нет**; права `id-token`+`issues: write`; `share: false`                                                                       |
| `opencode-review.yml`    | фильтр по автору PR есть; права `id-token`+`contents: read`+`pull-requests: write`+**`issues: write` (лишний)**; `share: false`             |
| `opencode-scheduled.yml` | события schedule/dispatch (автора нет); права id-token+contents+pull-requests+issues write; `share: false`; timeout 20 + concurrency (main) |
| Инцидент-доказательство  | триаж бот-issue #26: прогон 37757599684 failure на perm-гейте + мусорный комментарий бота                                                   |
| Сайд-канал расписания    | PR #27 (report.json +9117) от output-канала экшена — закрыт; канал задокументирован в task-11 step-2 report                                 |
| `opencode.json`          | `permission.external_directory` `/home/runner/*` allow (PR #29), MCP-серверы render/playwright/chrome-devtools                              |
| README                   | раздела про агентные воркфлоу нет; есть абзац CI (строки 137–140)                                                                           |
| Секреты                  | `HEXLET_ID`, `ZHIPU_API_KEY` (`gh secret list`); переменная `APP_URL` задана                                                                |

## Требования → шаги (description.md)

| # | Требование                                                                                    | Шаг  |
| - | --------------------------------------------------------------------------------------------- | ---- |
| 1 | Во всех воркфлоу агента условие, отсекающее события от ботов и от самого приложения           | 1, 3 |
| 2 | Набор команд задан явно через `mentions` либо в README записано, почему дефолт                | 1, 2 |
| 3 | Права выданы по воркфлоу отдельно; write только там, где агент создаёт коммиты, задачи или PR | 1, 3 |
| 4 | В README описаны воркфлоу: чем запускаются, какая модель, где смотреть прогоны                | 2    |
| 5 | Для публичного репозитория решение по `share` принято и записано                              | 1, 2 |
| 6 | В README или в задаче короткая самооценка: что с первого прохода, где итерации                | 2    |
| 7 | Круг тех, кто может звать агента, ограничен (публичный репозиторий)                           | 1, 2 |

Задачи описания → шаги: (24) триггеры/боты → 1 (правка triage, проверка
остальных), 3; (25) права, убрать лишние → 1 (review), 3; (26) таблица
в README → 2; (27) решение по share → 1 (явный false), 2 (запись);
(28) круг вызывающих → 1 (author_association), 2 (запись);
(29) самооценка → 2.

## Шаги

Все файлы шагов — в `docs/context/tmp/task-12/`, имена с префиксом
`20261008213145-` (опущен в таблице).

| Шаг | Файл                                | Суть                                                                              | Артефакт                               |
| --- | ----------------------------------- | --------------------------------------------------------------------------------- | -------------------------------------- |
| 1   | `step-1-workflow-guards-to-main.md` | бот-фильтр triage; `mentions`/`share`/круг в opencode.yml; права review; PR `ci:` | три воркфлоу в `main`, активны         |
| 2   | `step-2-readme-process-docs.md`     | README: таблица 9 воркфлоу, решения, самооценка; PR `docs:`                       | раздел в `main`                        |
| 3   | `step-3-acceptance.md`              | приёмка семи требований: grep + живые тесты + опц. dispatch                       | таблица приёмки, материал cycle-report |

## Global Constraints

- Правки строго в трёх файлах: `.github/workflows/opencode-triage.yml`,
  `.github/workflows/opencode.yml`, `.github/workflows/opencode-review.yml`
  (+ README.md в шаге 2). `opencode-scheduled.yml`,
  `hexlet-check.yml` и предсуществующие CI-воркфлоу не изменяются.
- Новые события (`on:`) не добавляются; структура jobs не меняется —
  только `if`-условия, `permissions` (только понижение `issues`),
  параметры `mentions`/`share` шага `Run OpenCode`.
- Бот-условие — по `user.type != 'Bot'`; ограничение вызывающих — по
  `author_association` (OWNER/COLLABORATOR/MEMBER); без обращений к
  внешним API в условии.
- Команды вызова после шага 1 — только `/oc`; строка `'/opencode'`
  из `if` удаляется.
- `share: false` во всех четырёх агентных воркфлоу, явной строкой
  (не комментарием).
- Коммиты — Conventional Commits: воркфлоу — `ci:`, README — `docs:`;
  изменения в `main` — только через PR, мерж `--merge`.
- Значения секретов не проходят через чат, логи и файлы — только маски
  (no-secrets-exposure).
- В README-таблице модель указывается в формате `provider/model`
  (`zai-coding-plan/glm-5.3-flash`) — как в воркфлоу.

## Review Focus (что скорее всего укусит)

1. **Ветка `task-12` отстала от `main`** — база `4689783` не содержит
   hardening PR #29–32 и релиза 0.4.0; правки поверх старой базы
   принесут в PR чужой обратный дифф. Ожидание: `git merge --ff-only
   origin/main` на старте шага 1; `git diff origin/main...HEAD` в PR —
   только целевые файлы. Пин — шаг 1 (предусловие).
2. **`issues: read` в review может потерять чтение issues** — при явном
   permissions-блоке незаданные скоупы = none; если ревью-прогон упадёт
   с 403 на issue-API, вернуть `issues: write` докоммитом `ci:`
   (вероятность мала: prompt пишет только комментарий к PR). Пин —
   шаг 3 (наблюдение за следующим ревью-прогоном).
3. **`/opencode` больше не вызывает агента** — по привычке можно
   написать `/opencode` и не получить ответа; прогон будет `skipped`.
   Ожидание: README фиксирует единственную команду `/oc`. Пин — шаг 2
   (запись), шаг 3 (живой skipped-тест).
4. **Посторонний комментатор** — живая негативная проверка невозможна
   (нет второго аккаунта); `author_association` у чужих комментариев —
   `NONE` → `skipped`. Доказательство: code-reading условия + docs
   GitHub event payload. Пин — шаг 3 (маркировка code-reading).
5. **Ревью собственного ci:-PR** — после мержа шага 1
   `opencode-review` сработает на оба PR задачи (автор — человек,
   фильтр проходит). Это ожидаемый конвейер; замечания ревью к самому
   воркфлоу — материал докоммита, мерж не блокировать (прецедент
   task-11 step-1). Пин — шаги 1–2 (наблюдение).
6. **`share: false` убирает ссылки `opencode.ai/s/…` из ответов** —
   доказательства прогонов теперь в логах Actions и тексте комментариев,
   не в shared-сессиях. Пин — шаг 3 (проверка отсутствия ссылки).
7. **У schedule/dispatch нет автора события** — бот-условие к ним
   неприменимо; защита — timeout + concurrency (уже стоят в `main`).
   В приёмке это отражается отдельно и не считается провалом
   требования 1. Пин — шаг 3.

## Открытые вопросы

1. Anti-spam для автотриажа от незнакомцев (фильтр возраста аккаунта
   ≥ 30 дней — docs Issues Triage Example) — спека не требует; триаж
   дешёвый, вопрос отложен.
2. Сайд-канал scheduled-прогона (PR с `report.json`) — закрывается
   вручную; варианты (удалять файл до шага агента, вопрос в upstream
   экшена) — вне минимальной спеки.
3. Фильтр возраста аккаунта для `/oc`-вызовов (сейчас author_association)
   — достаточно текущего; пересмотр при первом реальном спаме.
4. Финальное место контекстов — переезд `docs/context/tmp/task-12/` →
   `docs/context/task-12/` после приёмки (образец task-8–11), отдельным
   `docs:`-коммитом, в шаги не входит.

## Критерии приёмки (description.md, «Требования» + «Результат шага»)

- В `main` у `opencode-triage.yml` есть `if` с `user.type != 'Bot'`;
  у `opencode.yml` и `opencode-review.yml` бот-условия сохранены
  (требование 1; для scheduled — n/a, задокументировано).
- В `opencode.yml` задан `mentions: /oc`; причина сужения записана
  в README (требование 2).
- Права по воркфлоу раздельные; в review нет write на issues;
  write-набор scheduled обоснован в README (требование 3).
- В README есть таблица девяти воркфлоу: событие, модель, назначение,
  где смотреть прогоны (требование 4).
- Решение по `share` принято (false) и записано: явная строка во всех
  агентных воркфлоу + абзац в README (требование 5).
- В README есть короткая самооценка: с первого прохода и с итерациями,
  с опорой на номера прогонов/PR (требование 6).
- В `opencode.yml` условие `author_association` ограничивает круг
  вызывающих; правило записано в README (требование 7).
- Результат шага: процесс устойчив — петель нет, права минимальные,
  расходы под контролем, решения записаны в репозитории.

## Источники выводов

- real-run (fetch, 2026-10-08): opencode.ai/docs/github — Configuration
  (`mentions`: «Comma-separated list of trigger phrases, case-insensitive.
  Defaults to /opencode,/oc»; `share`: «Defaults to true for public
  repositories»; `model` обязателен, формат provider/model; `variant`:
  «Model variant for provider-specific reasoning effort»; `agent`:
  «Must be a primary agent»; `use_github_token` — режим без App),
  Schedule Example (write-набор прав обязателен: «Scheduled workflows
  run without a user context to permission-check»), Pull Request Example
  (read-набор: contents/pull-requests/issues read + write только
  pull-requests при комментировании), Issues Triage Example (фильтр
  возраста аккаунта ≥ 30 дней как anti-spam); opencode.ai/docs/permissions
  — значения allow/ask/deny, дефолты (read allow, .env deny,
  external_directory/doom_loop ask), права по агенту.
- real-run (shell, 2026-10-08): `gh repo view --json visibility`
  (PUBLIC); `gh workflow list` (9 активных); `git log`/`git status`/
  `git branch` (main @ d54aae7; локальная ветка task-12 @ 4689783,
  дерево чистое, уникальных коммитов нет); `git diff origin/main HEAD
  -- .github/workflows/*` (triage и review идентичны main; opencode.yml
  отличается только timeout 20↔45; scheduled — отсутствием
  timeout/concurrency); `gh api repos/:owner/:repo` (.private=false,
  default_branch=main); чтение файлов: все четыре агентных воркфлоу,
  README.md, .github/workflows/README.md, opencode.json (в чекауте
  task-12 без permission-блока; в main блок есть — PR #29).
- real-run (архив task-8–11, чтение отчётов 2026-10-06…08):
  cycle-report task-8 (самопробуждение на подстроке /opencode, 4
  AuthError Zen, фикс PR #6, perm-гейт для бот-комментариев);
  cycle-report task-9 (skipped 1s на комментарии без /oc, share: false
  подтверждён); step-4-report task-10 (rebase агента — две попытки,
  --force-with-lease; release-please — три инфраструктурных фикса;
  GITHUB_TOKEN-PR не триггерит pull_request-воркфлоу — 37464660414
  action_required); step-2-report task-11 (зависание 37750627398 55+ мин
  → timeout/concurrency; сайд-канал PR #27; триаж #26 — failure
  37757599684 на бот-авторе).
- code-reading: условие `author_association` отсекает посторонних до
  старта job (GitHub вычисляет `if` до запуска; skipped-прогон не
  тратит минуты) — docs GitHub workflow syntax; подстрока `'/oc'` не
  встречается в URL opencode.ai (в отличие от `'/opencode'`) — разбор
  инцидента task-8; незаданные скоупы при явном permissions-блоке = none
  — docs GitHub permissions; для schedule/dispatch фильтр по автору
  неприменим (у события нет автора) — структура event payload.
