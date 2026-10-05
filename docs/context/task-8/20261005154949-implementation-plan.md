# implementation-plan

# TASK 8 — агент OpenCode в GitHub Actions: план имплементации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for
> tracking.

- Дата: 2026-10-05 15:49
- Ветка: план для рабочей ветки `task-8` (уже создана от `main`, сейчас в ней)
- Шаг: план реализации (task-8)
- Spec: `docs/context/tmp/task-8/description.md`

**Goal:** подключить агента OpenCode к репозиторию `ai-for-developers-
project-387` через GitHub Actions: GitHub App, воркфлоу `opencode.yml` в
`main`, ключ модели в секретах, и живое доказательство — ответ агента на
`/oc explain this issue` в issue с прогоном в Actions.

**Architecture:** новая инфраструктура процесса, не продуктовый код.
Триггеры — `issue_comment` и `pull_request_review_comment` с
`types: [created]`; фильтр по команде (`/oc`, `/opencode`) — условием `if`
на job; аутентификация агента — OIDC (`id-token: write`) против
установленного GitHub App, ключ модели — только в GitHub Secrets.
Продуктовая функциональность (backend/frontend) не меняется вообще.

**Tech Stack:** GitHub Actions (`anomalyco/opencode/github@latest`),
GitHub App «opencode-agent», OpenCode Zen (`opencode/glm-5.3-flash` —
модель провайдера `opencode`, ключ куратора «Hexlet Students»), `gh` CLI.

## Задача

`description.md` (task 8 курса): агент перестаёт быть локальным
инструментом и запускается в GitHub Actions по событию репозитория.
Три части настройки: GitHub App, файл воркфлоу, ключ провайдера в
секретах. Результат шага: «воркфлоу лежит в основной ветке, ключ в
секретах, и вызов из issue-комментария подтверждён ответом агента и
прогоном в Actions».

## Ключевые решения

- **Модель — `opencode/glm-5.3-flash`** (OpenCode Zen, $0.15/$0.50 за
  1M токенов): ключ для GitHub Actions у студента программы «ИИ для
  разработчиков» выдаёт куратор пространства Hexlet Students, и он
  работает только с моделями `opencode/…` → секрет называется
  `OPENCODE_API_KEY` (справка Хекслета, real-run fetch 2026-10-05).
  Дешёвая flash-модель достаточно для `explain this issue`; точный id
  сверяется по `/models` в момент выполнения шага 3.
- **Секрет набирает пользователь, агент значения не видит**: ключ
  приходит от куратора в личные сообщения и вводится самим пользователем
  через `gh secret set` (интерактивный ввод) или web-UI; агент проверяет
  факт секрета по `gh secret list` (имя и дата, без значения).
- **Воркфлоу — по официальной документации OpenCode** (manual setup,
  real-run fetch 2026-10-05), с двумя заменами: env-переменная
  `OPENCODE_API_KEY` вместо `ANTHROPIC_API_KEY` и модель
  `opencode/glm-5.3-flash` вместо `anthropic/…`. Права минимальные:
  только `id-token: write`; checkout — `persist-credentials: false`.
- **Через PR, не прямой push в main**: требование спеки — «воркфлоу
  замержен в основную ветку»; PR даёт прогон CI на ветке и merge-коммит
  (конвенция task-7: PR + `gh pr merge --merge`).
- **Issue для вызова агента — из плана развития** task-7-pre
  (`20261005142605-development-plan.md`, пункт «Баг 1: дубликаты»):
  по замыслу курса следующие уроки гоняют агента именно по этим задачам;
  формулировка переносится в issue без изменения сути.

## Механика

- Шаг 1 — ветка `task-8`, PR → `main`, мерж; шаги 2–3 — после мержа,
  в `main` (шаг 2 коммитов не даёт, шаг 3 — только issue и комментарии).
- Коммиты — Conventional Commits; тип коммита воркфлоу — `ci:`.
- `docs/context/tmp/task-8/` — рабочий каталог; финальное место контекстов
  цикла решит пользователь по образцу task-7 (`docs/context/task-8pre/`
  или иное) — в объём шагов не входит.
- Секреты: значение ключа не проходит через чат, логи и файлы репозитория
  (no-secrets-exposure); в отчётах — только маска `OPENCODE_API_KEY=***`.
- Существующие воркфлоу (`backend-ci`, `frontend-ci`, `hexlet-check`,
  `security`, `release-please`) не изменяются; `.github/workflows/
  README.md` описывает только hexlet-check и не трогается.
- Продуктовый код, контракт, Docker, README — вне объёма: спека «Проверяется
  процесс. Продуктовая функциональность не растёт».

## Текущее состояние (real-run 2026-10-05)

| Что                | Состояние                                                                                                                         |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| Ветка / дерево     | ветка `task-8` от `main` (`2dbe5d9`), upstream нет; в дереве правка пользователя description.md task-7pre — в коммиты не включать |
| CI на `main`       | backend-ci, frontend-ci, hexlet-check, release-please — success на `2dbe5d9`                                                      |
| Воркфлоу агента    | `.github/workflows/opencode.yml` отсутствует (5 предсуществующих yml)                                                             |
| Секреты            | только `HEXLET_ID` (2026-10-01); `OPENCODE_API_KEY` нет (`gh secret list`)                                                        |
| GitHub App         | CLI-проверка установки недоступна (403); ставит пользователь в браузере, доказательство — прогон шага 3                           |
| Трекер             | 1 issue: #1 «No-auth admin mutations…» (OPEN, label `documentation`)                                                              |
| План развития      | task-7pre `20261005142605-development-plan.md`: 2 фичи + 1 баг, баг подтверждён real-run                                          |
| Локальный OpenCode | auth: `zai` + консоль opencode (`auth.json`); версия v2 — курсовая инструкция применима                                           |

## Требования → шаги (description.md)

| # | Требование                                                                        | Шаг    |
| - | --------------------------------------------------------------------------------- | ------ |
| 1 | GitHub App с агентом установлен на репозиторий проекта                            | 2 (+3) |
| 2 | Воркфлоу агента в `.github/workflows/` замержен в основную ветку                  | 1      |
| 3 | События ограничены `types: [created]`, работа — условием по команде в комментарии | 1      |
| 4 | Права `id-token: write`, checkout с `persist-credentials: false`                  | 1      |
| 5 | Ключ провайдера в GitHub Secrets, в файлах репозитория его нет                    | 2      |
| 6 | Issue с ответом агента на команду и прогоном этого вызова в Actions               | 3      |

Задачи описания → шаги: (1) App + файл воркфлоу → 1–2; (2) ключ в
секреты + лимит расходов → 2; (3) разбор воркфлоу (события, условие,
права, модель) → 1; (4) issue + `/oc explain this issue` → 3; (5) разбор
прогона Actions (время, логи шага агента) → 3.

## Шаги

| Шаг | Файл                                             | Суть                                                  | Артефакт                                        |
| --- | ------------------------------------------------ | ----------------------------------------------------- | ----------------------------------------------- |
| 1   | `20261005154949-step-1-workflow-to-main.md`      | файл воркфлоу, YAML, PR → `main`, мерж, разбор        | воркфлоу в `main`, разбор, CI зелёный           |
| 2   | `20261005154949-step-2-app-secret-limit.md`      | App + секрет `OPENCODE_API_KEY`, лимит (пользователь) | App установлен, секрет есть, ключа в файлах нет |
| 3   | `20261005154949-step-3-agent-call-acceptance.md` | issue, `/oc explain this issue`, прогон, приёмка      | ответ агента, прогон Actions, таблица приёмки   |

Шаги линейные: 1 → 2 → 3. Воркфлоу для `issue_comment` берётся из
дефолтной ветки, поэтому шаг 3 невозможен до мержа шага 1; вызов агента
(шаг 3) невозможен до App+секрета (шаг 2).

## Global Constraints

- События воркфлоу — `issue_comment` и `pull_request_review_comment`,
  оба строго `types: [created]` (description.md).
- Запуск работы — условием по команде в тексте комментария: `contains
  (github.event.comment.body, '/oc') || contains(…, '/opencode')`
  (description.md, docs/github).
- Права воркфлоу минимальные: только `id-token: write`; `actions/checkout`
  с `persist-credentials: false` (description.md).
- Ключ — только в GitHub Secrets (`OPENCODE_API_KEY`); в файле воркфлоу,
  AGENTS.md, документации и коммитах его быть не должно (description.md,
  no-secrets-exposure).
- Коммиты — Conventional Commits: `feat:`, `fix:`, `chore:`, `ci:`,
  `docs:` (AGENTS.md).
- Сгенерированное руками не правится (AGENTS.md) — к воркфлоу не
  применяется, файл пишется целиком руками: это конфигурация, не артефакт
  генерации.
- `hexlet-check.yml` не трогать (README `.github/workflows/`, AGENTS.md).
- rust/node пины не меняются (AGENTS.md; вне объёма этой задачи).

## Review Focus (что скорее всего укусит)

1. **Секрет отсутствует или пуст** — воркфлоу стартует, но шаг агента
   падает на аутентификации (пустой секрет роняет запуск — справка
   Хекслета). Ожидание: упавший прогон с ошибкой auth, а не тишина.
   Пин — шаг 3 (таблица «Проверка», пункт прогон success) и шаг 2
   (`gh secret list` до вызова).
2. **App не установлен на репозиторий** — OIDC-обмен не пройдёт, шаг
   агента упадёт уже после checkout. CLI-проверки установки нет (403,
   real-run 2026-10-05), поэтому установка — действие пользователя в
   браузере, а доказательство — успешный прогон шага 3. Пин — шаги 2–3.
3. **Ключ утёк в файлы репозитория** — инцидент даже в приватном репо:
   остаётся в истории после удаления (description.md). Ожидание: grep по
   файлам репо не находит значения ключа; значение вводится только через
   `gh secret set`/web-UI. Пин — шаг 2 (grep) и правило масок в отчётах.
4. **Комментарий без команды запускает лишнюю работу** — воркфлоу
   стартует на каждый новый комментарий (событие), работа фильтруется
   `if`; слово `/oc` внутри произвольного текста (например, URL) тоже
   вызовет агента. Ожидание: это документированное поведение docs, а не
   дефект; проверяется в шаге 3 разбором условия. Пин — шаг 1 (разбор).
5. **Правка старого комментария будит агента** — без `types: [created]`
   событие срабатывает и на edited; требование спеки фиксирует
   `types: [created]` в обоих событиях. Пин — шаг 1 (разбор + grep).
6. **Модель недоступна ключу куратора** — ключ работает только с
   `opencode/…`; чужой `anthropic/…` в `model:` даст ошибку провайдера на
   живом прогоне. Пин — шаг 1 (значение `model:`) и шаг 3 (прогон).
7. **Грязное дерево перед шагом 1** — в дереве лежит правка пользователя
   `docs/context/task-7pre/description.md`; коммит воркфлоу должен
   содержать только `.github/workflows/opencode.yml` (точечный
   `git add`). Пин — шаг 1 (предусловие и действие коммита).

## Открытые вопросы

1. **Точный id модели** — `opencode/glm-5.3-flash` выбран по цене и
   совпадению с моделью текущей сессии; финальный выбор — за пользователем
   по `/models` пространства Hexlet Students в момент шага 3. Замена —
   правка одной строки `model:` тем же коммитом `ci:`.
2. **Источник ключа** — у студента программы «ИИ для разработчиков» ключ
   для Actions выдаёт куратор (вкладки API Keys в Hexlet Students нет).
   Если пользователь решит использовать личное пространство — ключ
   создаётся в opencode.ai/console, шаг 2 не меняется, меняется только
   кто ключ выдал; лимит расходов тогда ставит пользователь сам.
3. **Лимит расходов** — в Hexlet Students общий лимит $20 уже задан
   программой ($15/мес терминал + остаток на ключе Actions); действие
   шага 2 — проверить Overview в консоли. Для личного пространства —
   выставить monthly usage limit. Пользовательское действие, агент
   верифицировать его не может.
4. **Финальное место контекстов цикла** — `docs/context/tmp/task-8/`
   сейчас; переезд по образцу task-7-pre → task-7pre — решение
   пользователя после приёмки, отдельным `docs:`-коммитом, в шаги не
   входит.
5. **Правка `docs/context/task-7pre/description.md` в дереве** —
   пользовательская (добавлен раздел «Что дальше»); не входит в коммиты
   цикла; судьба правки — решение пользователя.

## Критерии приёмки (description.md, «Требования» + «Результат шага»)

- GitHub App «opencode-agent» установлен на репозиторий (подтверждение
  пользователя + успешный OIDC-прогон шага 3).
- `.github/workflows/opencode.yml` существует и замержен в `main`
  (merge-коммит PR в истории `main`).
- Оба события — `types: [created]`; работа — за условием `if` по
  `/oc`/`/opencode`; права — только `id-token: write`; checkout —
  `persist-credentials: false` (разбор шага 1).
- `OPENCODE_API_KEY` в GitHub Secrets (`gh secret list`); значения ключа
  нет ни в одном файле репозитория (grep).
- Лимит расходов проверен/выставлен в кабинете (действие пользователя).
- Существует issue, где агент ответил на `/oc explain this issue`;
  соответствующий прогон виден во вкладке Actions с длительностью и
  логами шага агента (шаг 3).
- Предсуществующие воркфлоу зелёные на `main` после мержа шага 1.

## Источники выводов

- real-run (fetch, 2026-10-05): opencode.ai/docs/github — manual setup,
  YAML воркфлоу, параметры (`model`, `mentions`, `share`, `use_github_token`),
  события и примеры; opencode.ai/docs/zen — список моделей `opencode/…`,
  цены (GLM 5.3 Flash $0.15/$0.50), лимиты; help.hexlet.io/ai/opencode —
  секрет `OPENCODE_API_KEY` для `opencode/…`, ключ от куратора Hexlet
  Students, «пустой секрет роняет запуск», лимит $20.
- real-run (shell, 2026-10-05): `git branch --show-current` → `task-8`;
  `git status --porcelain` → правка `docs/context/task-7pre/description.md`;
  `git log --oneline -3` (`2dbe5d9` head task-8); `gh run list` (CI success
  на `main`); `gh secret list` (только `HEXLET_ID`);
  `gh api /repos/…/installation` → 401 и `/user/installations` → 403
  (CLI-проверки установки App нет); `gh issue list` → issue #1 OPEN;
  `ls .github/workflows/` (5 файлов, opencode.yml нет); `python3 -c
  "import yaml"` (pyyaml есть, actionlint/yamllint не установлены);
  `cat ~/.local/share/opencode/auth.json` → провайдеры `234`, `zai`
  (без вывода значений).
- real-run (чтение файлов, 2026-10-05): `docs/context/tmp/task-8/
  description.md` (требования, задачи, результат); `docs/context/
  task-7pre/20261005142605-development-plan.md` (баг 1 — источник issue);
  `docs/context/task-7pre/20261005150410-cycle-outcome.md` (канонические
  пути, конвенция мержа); `.github/workflows/README.md` (только
  hexlet-check); `docs/agents/issue-tracker.md` (gh-конвенции);
  `~/ai-rules/mattpocock/skills` — writing-plans/executing-plans (формат
  планов, совпадает с образцом task-7pre).
- code-reading: порядок шагов — из семантики `issue_comment` (воркфлоу
  берётся из дефолтной ветки) и зависимости «вызов агента ← App + секрет»;
  тип коммита `ci:` — из классификации Conventional Commits.
