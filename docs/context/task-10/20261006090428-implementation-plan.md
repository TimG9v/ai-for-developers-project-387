# implementation-plan

# TASK 10 — от issue к pull request и ревью: план имплементации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for
> tracking.

- Дата: 2026-10-06 09:04 (+05)
- Ветка: шаг 2 — рабочая ветка `task-10` (создаётся от свежего `main` в
  момент старта шага); шаги 1, 3, 4 — без коммитов кода и файлов плана
  (работа через GitHub: issue, PR агента, мерж)
- Шаг: план реализации (task-10)
- Spec: `docs/context/tmp/task-10/description.md` (в запросе пользователя
  файл назван `descripton.md` — фактическое имя `description.md`)

**Goal:** пройти полный цикл issue → PR → ревью → правки → мерж с
участием агента: агент готовит PR по разобранной задаче (issue #7),
человек оставляет оба вида замечаний (общий комментарий и к строке
diff), агент правит ту же ветку, авторевью-воркфлоу на `pull_request`
оставляет свои замечания, после мержа release-please собирает
release-PR.

**Architecture:** изменений в репо ровно одно — новый файл
`.github/workflows/opencode-review.yml` (авторевью на `pull_request` с
четырьмя типами, `use_github_token: true`, `prompt` с критериями).
Продуктовый код в цикле пишет сам агент в ветке `opencode/*` по issue
#7 — план не пишет продуктовый код. Существующий `opencode.yml` уже
слушает `issue_comment` и `pull_request_review_comment` — оба канала
человеческого ревью работают без правок воркфлоу.

**Tech Stack:** GitHub Actions (`anomalyco/opencode/github@latest`),
модель `zai-coding-plan/glm-5.3-flash` + секрет `ZHIPU_API_KEY`
(рабочая связка task-8), `gh` CLI, release-please
(`release-please.yml`, manifest `0.1.0`).

## Задача

`description.md` (task 10 курса): проходится полный путь «от issue к
pull request и ревью». Агент готовит PR по разобранной задаче; человек
проводит ревью двумя каналами — общий комментарий приходит событием
`issue_comment` (PR технически issue), комментарий к строке diff —
`pull_request_review_comment` (агент получает путь файла, номер строки
и окружающий diff, указывать их в тексте не нужно). Третий сценарий —
авторевью: воркфлоу на `pull_request` (opened, synchronize, reopened,
ready_for_review), `prompt` с собственными критериями, замечания —
токеном раннера (`use_github_token: true`), прав хватает на чтение;
мержит и одобряет человек. После мержа release-please собирает
release-PR. Результат: цикл «issue → pull request → ревью → правки →
мерж» пройден с участием агента, авторевью работает на каждый PR,
release-please собирает release-PR.

## Ключевые решения

- **Issue для цикла — #7** («Отмена и перенос записи»): в треде есть
  план исправления и постановка «что считается исправлением» (task-9).
  «В том же issue» = issue, разобранный на предыдущем шаге курса. #9
  не годится — ждёт ответов на 4 вопроса агента (needs-info), чинить
  нечего. #5 и #1 без постановки для исправления.
- **`/oc`-команда с явными требованиями к PR** (Closes #7, Conventional
  Commits, минимум один `feat:`, генерацию не править руками): агент
  читает тред и AGENTS.md, но дублирование в команде делает исход
  проверяемым — `Closes #7` закрывает требование 1 (связь с issue),
  `feat:` — условие работы release-please (`chore:` не даёт бампа),
  запрет ручных правок генерации — критерий авторевью.
- **Воркфлоу авторевью — в `main` ДО пуша правок агента** (шаг 2 между
  шагами 1 и 3): тогда `synchronize` feature-PR триггерит авторевью на
  богатом diff, а не только reopen release-PR. Отступление от порядка
  задач описания (воркфлоу там пятым пунктом) осознанное: требования
  порядок не фиксируют, а доказательная ценность выше.
- **Отдельный файл `opencode-review.yml`, не правка `opencode.yml`**:
  другое событие, права, токен-режим; конвенция task-8/9 —
  предсуществующие воркфлоу не трогаются. Имя — как в docs-примере.
- **Права — ровно как в docs Pull Request Example + спека**:
  `id-token: write` + `contents: read` + `pull-requests: read` +
  `issues: read`, `use_github_token: true`, `GITHUB_TOKEN` в env.
  Известный риск: постинг комментариев через `GITHUB_TOKEN` по семантике
  разрешений GitHub требует write — живой прогон на ci:-PR проверит;
  fallback — докоммит `pull-requests: write` + `issues: write` одним
  `ci:`-коммитом (паттерн task-9 «докоммит одной строки»).
- **`share: false` в авторевью**: комментарии бота без ссылки на
  shared-сессию не содержат `//opencode` и не самопробуждают
  `opencode.yml` — подтверждено task-9 (триаж).
- **Секвентность ревью-комментариев**: общий → дождаться прогона и пуша
  → строчный. Два параллельных прогона агента пушат в одну ветку —
  риск отказа пуша/конфликта.
- **Через PR, не прямой push в `main`**; `gh pr merge --merge` —
  merge-коммит сохраняет индивидуальные коммиты агента, а release-please
  разбирает именно их (при squash разбирался бы заголовок сквоша).
- **Release-PR не мержить**: спека требует «найдите release-PR» — его
  сборка и есть результат; мерж релиза — за пределами шага.
- **Reopen release-PR — опциональное доказательство**: PR, созданный
  `GITHUB_TOKEN`, не триггерит `pull_request`-воркфлоу при открытии
  (документированное поведение GitHub Actions); закрыть+переоткрыть
  руками → событие `reopened` (в списке типов) → прогон. Требование 4
  закрывается и без этого (ci:-PR на `opened` + feature-PR на
  `synchronize`), reopen — полное покрытие «на новом pull request».

## Механика

- Шаг 1 — `/oc`-команда в #7, агент создаёт ветку `opencode/*` и PR,
  проверка PR. Шаг 2 — ветка `task-10`, файл `opencode-review.yml`,
  PR → `main`, мерж, живая проверка на собственном ci:-PR. Шаг 3 —
  ревью-цикл: общий и строчный `/oc`-комментарии, правки агента в ту же
  ветку, synchronize-авторевью. Шаг 4 — формат коммитов, мерж
  feature-PR, release-PR, приёмка.
- Линейность 1 → 2 → 3 → 4: шаг 3 требует авторевью в `main`
  (synchronize-доказательство), шаг 4 требует учёта правок из шага 3.
- Расход: ~6 модельных прогонов (`/oc fix`, 2 ревью, 2–3 авторевью) +
  `skipped`-самопробуждения коммент-воркфлоу (без расхода, паттерн
  task-8/9: failure на perm-гейте → skipped, цикл затухает).
- `docs/context/tmp/task-10/` — рабочий каталог; переезд в
  `docs/context/task-10/` — решение пользователя после приёмки (образец
  task-8/9), в объём шагов не входит.
- Секреты: значение `ZHIPU_API_KEY` не проходит через чат, логи и файлы
  репозитория (no-secrets-exposure); в отчётах — только маски.
- Cycle-report — отдельный контекст-файл после приёмки (по образцу
  task-8/9); шаг 4 готовит для него материал (номера PR/прогонов,
  хронология, таблица приёмки).

## Текущее состояние (real-run 2026-10-06 09:04 +05)

| Что                   | Состояние                                                                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ветка / дерево        | `main` @ `94441cd` «docs: add task-9 cycle report and execution ledger», дерево чистое                                                            |
| CI на `main`          | backend-ci, frontend-ci, hexlet-check, Release Please — success (прогоны 37410984640/657/670, 37410984833)                                        |
| `opencode.yml`        | `issue_comment` + `pull_request_review_comment`, types `[created]`, `/oc`-гейт, `id-token: write`; active (id 375442803)                          |
| `opencode-triage.yml` | в `main` (task-9), `issues` `[opened]`, active (id 375954644)                                                                                     |
| `opencode-review.yml` | отсутствует (7 yml в `.github/workflows/` + README.md)                                                                                            |
| release-please        | `release-please.yml` (push → `main`), `googleapis/release-please-action@v4`, manifest `0.1.0`, config `simple`; release-PR ни разу не создавался  |
| PR                    | #2–#8 все MERGED/CLOSED, открытых нет; PR от агента в этом репо ещё не создавался                                                                 |
| Issues                | #7 (план исправления + постановка), #9 (ждёт ответов на 4 вопроса), #5, #1 — все OPEN                                                             |
| Секреты               | `HEXLET_ID` (2026-10-01), `ZHIPU_API_KEY` (2026-10-06) — `gh secret list`                                                                         |
| Perm-паттерн          | комментарии бота с share-ссылками самопробуждают `opencode.yml` → failure «does not have write permissions» → skipped (паттерн task-8, не дефект) |
| CI на PR              | backend/frontend-ci: push во все ветки + `pull_request` с paths-фильтром — PR агента получит CI                                                   |

## Требования → шаги (description.md)

| # | Требование                                                                           | Шаг       |
| - | ------------------------------------------------------------------------------------ | --------- |
| 1 | PR создан или обновлён агентом, связан с исходным issue                              | 1         |
| 2 | Оба вида замечаний: общий в обсуждении и к строке в diff                             | 3         |
| 3 | Агент внёс правки в ту же ветку, видно по коммитам                                   | 3         |
| 4 | Воркфлоу авторевью на `pull_request` с prompt, оставил замечания на PR               | 2 (+3,+4) |
| 5 | Коммиты агента по Conventional Commits; после мержа release-please собрал release-PR | 4         |

Задачи описания → шаги: (1) в том же issue попросить агента подготовить
PR → 1; (2) проверить PR: описание, изменения, связь с issue → 1;
(3) общий комментарий и комментарий к строке, в каждом позовать агента
→ 3; (4) агент обновил ту же ветку, учёл оба замечания → 3;
(5) воркфлоу авторевью + его замечания на новом PR → 2 (+3 synchronize,
+4 reopen); (6) формат коммитов, мерж, найти release-PR → 4.

## Шаги

Все файлы шагов — в `docs/context/tmp/task-10/`, имена с префиксом
`20261006090428-` (опущен в таблице).

| Шаг | Файл                                     | Суть                                                                          | Артефакт                                    |
| --- | ---------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------- |
| 1   | `step-1-agent-pr-from-issue.md`          | `/oc`-команда в #7 → ветка `opencode/*` + PR от агента; проверка PR           | PR связан с #7, коммиты `feat:`, CI зелёный |
| 2   | `step-2-auto-review-workflow-to-main.md` | `opencode-review.yml` (pull_request + prompt + use_github_token), PR → `main` | воркфлоу active, замечания на ci:-PR        |
| 3   | `step-3-review-fix-cycle.md`             | общий + строчный `/oc`-комментарии, правки агента, synchronize-авторевью      | 2 прогона success, коммиты правок в ветке   |
| 4   | `step-4-merge-release.md`                | формат коммитов, мерж feature-PR, release-PR, приёмка                         | merge-коммит, release-PR `0.2.0`, приёмка   |

## Global Constraints

- Новый файл строго `.github/workflows/opencode-review.yml`: событие
  `pull_request`, `types: [opened, synchronize, reopened,
  ready_for_review]`, непустой `prompt`, `use_github_token: true`,
  `GITHUB_TOKEN` в env, права `id-token: write` + `contents: read` +
  `pull-requests: read` + `issues: read` (description.md + docs Pull
  Request Example).
- Модель `zai-coding-plan/glm-5.3-flash`, env `ZHIPU_API_KEY` —
  единственная рабочая связка (task-8 путь B, real-run); значение ключа
  только в GitHub Secrets.
- Предсуществующие воркфлоу (`opencode.yml`, `opencode-triage.yml`,
  `release-please.yml`, CI, `hexlet-check.yml`) не изменяются;
  `hexlet-check.yml` не трогать.
- Коммиты — Conventional Commits: файл воркфлоу — `ci:`, контекст —
  `docs:` (AGENTS.md); изменения в `main` — только через PR с мержем
  `--merge` (сохранение индивидуальных коммитов — условие
  release-please).
- Продуктовый код в цикле пишет агент (ветка `opencode/*`); план не
  содержит правок продуктового кода.
- Сгенерированные артефакты (`frontend/src/client/`, серверные типы
  backend) не правятся руками — только `make generate` из `contracts/`
  (правило AGENTS.md и критерий авторевью).
- Release-PR после его появления не мержить (за пределами шага).

## Review Focus (что скорее всего укусит)

1. **Агент не может создать ветку/PR** — первый `/oc fix` в этом репо;
   по docs создание ветки и PR — core-поведение приложения, но права
   app-токена проверяются только живым прогоном. Ожидание: после
   success-прогона существует PR из `opencode/*`; иначе стоп-протокол
   шага 1 (не повторять команду вслепую). Пин — шаг 1.
2. **Авторевью на read-правах не может постить замечания (403)** —
   расхождение docs-примера с семантикой разрешений `GITHUB_TOKEN`.
   Ожидание: на ci:-PR есть замечание/резюме от `github-actions[bot]`;
   иначе докоммит `pull-requests: write` + `issues: write` следующим
   `ci:`-PR. Пин — шаг 2 (живой прогон).
3. **Параллельные прогоны от двух ревью-комментариев** пушат в одну
   ветку — отказ пуша/конфликт. Ожидание: строго секвентно — общий
   комментарий → success + пуш → строчный. Пин — шаг 3.
4. **Коммиты агента без `feat:`** — release-please не соберёт
   release-PR (требование 5 сорвано на самом видном месте). Ожидание:
   минимум один `feat:` среди коммитов PR, проверка до мержа. Пин —
   шаг 4 (проверка 1); профилактика — текст команды в шаге 1.
5. **Самопробуждение `opencode.yml`** от комментариев бота с
   share-ссылками — известный паттерн failure → skipped, не дефект;
   `share: false` в новом воркфлоу минимизирует. Пин — наблюдение в
   шагах 1–3, фиксация в cycle-report.
6. **Release-PR не триггерит авторевью при открытии** — создан
   `GITHUB_TOKEN`; ожидаемо, не дефект. Если нужно авторевью и на нём —
   close+reopen (`reopened`). Пин — шаг 4 (отрицательная проверка).
7. **Объём `/oc fix` (план из 7 шагов)** — риск таймаута или частичной
   реализации flash-моделью. Ожидание: PR может быть неполным — это
   рабочий материал ревью-цикла; fallback — вертикальный срез
   (контракт + backend) с дополнением через ревью. Пин — шаг 1.

## Открытые вопросы

1. **Полный объём фичи vs вертикальный срез первого PR** — решается по
   результату прогона `/oc fix` (шаг 1); ревью-цикл шага 3 легально
   достраивает недостающее.
2. **Reopen release-PR для авторевью** — опционально; решение
   пользователя в шаге 4 (требование 4 уже закрыто шагами 2–3).
3. **Если perm-проблема агента подтверждается** (шаг 1, стоп-протокол):
   (а) поднять права GitHub App installation (Settings → Integrations →
   opencode-agent), (б) `use_github_token: true` + `GITHUB_TOKEN` с
   write-правами в `opencode.yml` — правка существующего воркфлоу,
   отступление от конвенции, (в) агент готовит патч без пуша, коммитит
   человек. Решение пользователя, в шаги не предзаложено.
4. **Финальное место контекстов** — переезд `docs/context/tmp/task-10/`
   → `docs/context/task-10/` после приёмки (образец task-8/9),
   отдельным `docs:`-коммитом, в шаги не входит.

## Критерии приёмки (description.md, «Требования» + «Результат шага»)

- Существует PR, созданный агентом: ветка `opencode/*`, тело ссылается
  на #7 (Closes), среди коммитов есть `feat:`, CI на PR зелёный
  (требование 1).
- В PR оставлены оба вида замечаний с `/oc`-вызовом: общий в
  обсуждении и к строке в diff; оба прогона `opencode` — `success`
  (требование 2).
- После замечаний в той же ветке есть новые коммиты агента; дифф
  закрывает оба замечания (требование 3).
- `.github/workflows/opencode-review.yml` замержен в `main`: событие
  `pull_request` с четырьмя типами, непустой `prompt`,
  `use_github_token: true`, права как в docs; registered `active`; есть
  живой прогон с замечаниями/резюме на PR (требование 4).
- Коммиты агента в PR — по Conventional Commits; после мержа
  feature-PR release-please создал release-PR
  (`chore(main): release 0.2.0`) с changelog из feat-коммитов агента
  (требование 5).
- Цикл «issue → pull request → ревью → правки → мерж» пройден с
  участием агента; `main` после мержа зелёный; release-PR не смержен.

## Источники выводов

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — Supported
  Events (`pull_request`: авторевью на opened/synchronized/reopened;
  `pull_request_review_comment`: агент получает путь файла, номера строк
  и diff-контекст), Pull Request Example (`opencode-review.yml`:
  события, права, `GITHUB_TOKEN`, `use_github_token`, prompt),
  Configuration (`use_github_token`: GITHUB_TOKEN вместо OIDC-обмена;
  `share` default true для публичных репо), Examples (правки — в тот же
  PR; строчные комментарии — без указания файла/строки в тексте).
- real-run (shell, 2026-10-06): `git branch --show-current` /
  `git log --oneline -8` / `git status --porcelain` (main @ `94441cd`,
  чисто); `gh workflow list` (8 active, opencode-review нет);
  `gh issue list` (#7, #9, #5, #1 OPEN); `gh pr list --state all`
  (открытых нет, release-PR нет); `gh secret list` (HEXLET_ID,
  ZHIPU_API_KEY); `gh run list` (CI success); `gh issue view 7`
  (план + постановка); `cat .release-please-manifest.json` (0.1.0) и
  `release-please-config.json` (simple); grep триггеров
  backend/frontend-ci (push все ветки + pull_request paths).
- real-run (чтение файлов, 2026-10-06): `description.md` (требования,
  задачи, результат); `.github/workflows/opencode.yml` (уже слушает
  `pull_request_review_comment`; связка env/model), `release-please.yml`
  (push → main, дефолтные конфиги); `docs/context/task-8/
  20261006054241-cycle-report.md` (perm-гейт действия, путь B,
  самопробуждение); `docs/context/tmp/task-9/20261006063500-cycle-report.md`
  (судьба #7 и #9, подтверждение `share: false`).
- code-reading: порядок шагов 1 → 2 → 3 — pull_request-воркфлоу берётся
  из merge-ref PR, поэтому synchronize увидит воркфлоу из `main`;
  GITHUB_TOKEN-созданные PR не триггерят pull_request-события —
  документированное поведение GitHub Actions; 403-риск постинга
  замечаний на read-правах — семантика разрешений GitHub, проверяется
  живым прогоном шага 2; выбор #7 — из содержания треда (план +
  постановка) и итогов task-9; `feat:` → minor `0.1.0` → `0.2.0` —
  семантика release-please.
