# step-2-auto-review-workflow-to-main

- Дата: 2026-10-06 09:04 (+05)
- Ветка: `task-10` (создаётся от свежего `main` в начале шага);
  мерж через PR
- Шаг: план шага 2/4 — воркфлоу авторевью на событии `pull_request`
  с prompt и `use_github_token: true`, PR → `main`, мерж, живая
  проверка

Исходный план: `20261006090428-implementation-plan.md`.

## Цель

`.github/workflows/opencode-review.yml` существует в `main` (merge PR):
событие `pull_request` c `types: [opened, synchronize, reopened,
ready_for_review]`, непустой `prompt` с критериями ревью,
`use_github_token: true` + `GITHUB_TOKEN` в env, права `id-token:
write` + `contents: read` + `pull-requests: read` + `issues: read`;
воркфлоу зарегистрирован `active`; живое доказательство — прогон
авторевью на собственном ci:-PR с замечанием/резюме.

## Interfaces

- Consumes: docs Pull Request Example (структура файла); связка
  model/env из действующего `opencode.yml` (`ZHIPU_API_KEY` +
  `zai-coding-plan/glm-5.3-flash`, task-8 путь B); конвенция PR →
  `main` (task-7/8/9).
- Produces: воркфлоу авторевью в `main` — вход шага 3
  (synchronize-доказательство на PR агента) и шага 4 (reopen
  release-PR, опционально).

## Предусловия (real-run перед стартом)

- [ ] 1. Дерево чистое, `main` свежий, файла-авторевью нет:

  ```bash
  git checkout main && git pull && git status --porcelain
  ls .github/workflows/ | grep review
  ```

  Ожидание: `status` пуст; `grep` ничего не нашёл (exit 1).

- [ ] 2. PR агента из шага 1 существует и открыт (в нём шаг 3 получит
  synchronize-доказательство):

  ```bash
  gh pr list --state open
  ```

  Ожидание: PR из ветки `opencode/*` в списке.

## Действия

- [ ] 1. Создать рабочую ветку:

  ```bash
  git checkout -b task-10
  ```

- [ ] 2. (По решению пользователя) `docs:`-коммит контекст-файлов
  плана:

  ```bash
  git add docs/context/tmp/task-10
  git commit -m "docs: add task-10 context docs"
  ```

  Образец — task-9 (`e263a7f`).

- [ ] 3. Создать `.github/workflows/opencode-review.yml` целиком:

  ```yaml
  name: opencode-review

  on:
    pull_request:
      types: [opened, synchronize, reopened, ready_for_review]

  jobs:
    review:
      runs-on: ubuntu-latest
      permissions:
        id-token: write
        contents: read
        pull-requests: read
        issues: read
      steps:
        - name: Checkout repository
          uses: actions/checkout@v6
          with:
            fetch-depth: 1
            persist-credentials: false

        - name: Run OpenCode
          uses: anomalyco/opencode/github@latest
          env:
            ZHIPU_API_KEY: ${{ secrets.ZHIPU_API_KEY }}
            GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          with:
            model: zai-coding-plan/glm-5.3-flash
            share: false
            use_github_token: true
            prompt: |
              Проведи первое ревью этого pull request как участник команды
              календарного приложения. Критерии:
              1. Коммиты PR — по Conventional Commits (feat/fix/ci/docs/...).
              2. Контракт прежде всего: сгенерированные файлы
                 (frontend/src/client/, серверные типы backend) не правятся
                 руками — только перегенерация из contracts/main.tsp.
              3. Потенциальные баги: необработанные 404/409, гонки при
                 бронировании слота, валидация входных данных.
              4. Соответствие правилам AGENTS.md (стиль, типы, тесты).
              Оставь замечания комментарием к pull request: каждое —
              отдельным пунктом, с файлом и строкой. Если замечаний нет —
              короткое резюме «существенных замечаний нет».
              Не мержи и не одобряй PR.
  ```

- [ ] 4. Разбор файла — четыре осознанных отклонения/соответствия
  docs-примеру `opencode-review.yml`:

  - провайдер/модель/секрет: `zai-coding-plan/glm-5.3-flash` +
    `ZHIPU_API_KEY` вместо `anthropic/…` + `ANTHROPIC_API_KEY`
    (инцидент task-8: Zen-ключ vs coding-plan);
  - `share: false`: комментарии бота без ссылки на shared-сессию не
    содержат `//opencode` и не самопробуждают коммент-воркфлоу
    (подтверждено task-9);
  - `prompt` на русском с критериями из конвенций репо (Conventional
    Commits, контракт-first, AGENTS.md) — спека: «в prompt пишутся
    свои критерии ревью»; требование 4 — «с prompt»;
  - права и `use_github_token: true` — ровно как docs-пример и спека
    (read-набор + `id-token: write`); известный риск: постинг
    комментариев через `GITHUB_TOKEN` требует write — живой прогон на
    этом же PR проверит (fallback — докоммит write-прав, см. шаг 8).

- [ ] 5. Локальная проверка YAML:

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-review.yml')); print('YAML-OK')"
  grep -n "types: \[opened, synchronize, reopened, ready_for_review\]" .github/workflows/opencode-review.yml
  grep -n "use_github_token: true" .github/workflows/opencode-review.yml
  grep -n "prompt:" .github/workflows/opencode-review.yml
  ```

  Ожидание: `YAML-OK`; три grep-строки найдены (review focus 2 и
  требования спеки).

- [ ] 6. Точечный коммит (только новый файл):

  ```bash
  git add .github/workflows/opencode-review.yml
  git status --porcelain
  git commit -m "ci: add opencode auto-review workflow for pull requests"
  ```

  Ожидание: в индексе один файл; Conventional Commit `ci:`.

- [ ] 7. PR → `main`, дождаться CI, мерж:

  ```bash
  git push -u origin task-10
  gh pr create --title "ci: add opencode auto-review workflow for pull requests" --fill
  gh pr checks --watch
  gh pr merge --merge
  ```

  Ожидание: проверки зелёные. Внимание: среди проверок будет сам
  `opencode-review` — pull_request-воркфлоу берётся из merge-ref PR,
  поэтому на собственном ci:-PR он срабатывает на `opened` ещё до
  мержа. Это первый живой прогон авторевью — его замечание/резюме
  читается в PR до мержа; мерж не блокировать из-за замечаний к самому
  воркфлоу (они — материал докоммита при необходимости).

- [ ] 8. Регистрация воркфлоу и разбор живого прогона:

  ```bash
  git fetch
  git show origin/main:.github/workflows/opencode-review.yml >/dev/null && echo MAIN-OK
  gh workflow list | grep review
  gh run list --workflow opencode-review --limit 3
  ```

  Ожидание: `MAIN-OK`; строка `opencode-review … active`; прогон на
  ci:-PR (`opened`) `success`; в PR — замечание/резюме от
  `github-actions[bot]`.

  **Fallback** (прогон упал на 403 постинга комментария — review focus
  2): добавить `pull-requests: write` + `issues: write` в permissions,
  докоммитить `ci:` следующим PR, доказательство «оставил замечания»
  переносится на synchronize feature-PR (шаг 3). Отступление от
  read-набора спеки фиксировать в cycle-report с причиной (real-run
  403).

## Файлы

- Create: `.github/workflows/opencode-review.yml`.
- Modify: ничего; предсуществующие воркфлоу (включая `opencode.yml`,
  `opencode-triage.yml`, `release-please.yml`) не трогаются.
- Не трогать: код, контракт, README, `hexlet-check.yml`.

## Проверка (real-run)

| # | Команда                                                                                   | Ожидаемый результат                       |
| - | ----------------------------------------------------------------------------------------- | ----------------------------------------- |
| 1 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-review.yml'))"` | без исключения                            |
| 2 | `git show origin/main:.github/workflows/opencode-review.yml`                              | файл в `main`, содержимое = шаг 3         |
| 3 | `gh pr view --json state -q .state` (по URL ci:-PR)                                       | `MERGED`                                  |
| 4 | `gh workflow list \| grep review`                                                         | `opencode-review … active`                |
| 5 | `gh run list --workflow opencode-review --limit 1`                                        | `success`, event `opened`                 |
| 6 | комментарий в ci:-PR                                                                      | резюме/замечания от `github-actions[bot]` |

## Отрицательные проверки

- В файле нет значений секретов — только ссылки `${{ secrets.* }}`:

  ```bash
  grep -c "secrets\." .github/workflows/opencode-review.yml
  ```

  Ожидание: 2 (ZHIPU_API_KEY, GITHUB_TOKEN — обе ссылки, не значения).

- События файла — только `pull_request`:

  ```bash
  grep -n -A 3 "^on:" .github/workflows/opencode-review.yml
  ```

  Ожидание: без `issues` / `issue_comment` — дублей запусков с
  `opencode-triage` и `opencode` нет.

- Дифф PR против `main` — только новый файл (+ `docs:`-коммит
  контекстов, если взят):

  ```bash
  git diff origin/main...HEAD --name-only
  ```

## Коммит

Один коммит: `ci: add opencode auto-review workflow for pull requests`
(файл `.github/workflows/opencode-review.yml`); опционально
предшествующий `docs:`-коммит контекст-файлов. В `main` — через PR.

## Результат шага (description.md)

В `.github/workflows/` есть воркфлоу авторевью на событии
`pull_request` с prompt и критериями; живое подтверждение «оставил
замечания на pull request» — собственный ci:-PR здесь, synchronize
feature-PR — шаг 3, reopen release-PR — шаг 4 (опционально).

## Источники

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — Pull Request
  Example (`opencode-review.yml`: события с четырьмя типами, права
  read-набора + `id-token: write`, `GITHUB_TOKEN` в env,
  `use_github_token: true`, prompt с критериями), Configuration
  (`use_github_token`: caller-provided GITHUB_TOKEN вместо
  OIDC-обмена; `id-token` в этом режиме не требуется, в примере
  остаётся; `share` default true для публичных репо), Supported Events
  (`pull_request`: «Useful for automated reviews»; без prompt — ревью
  по правилам агента).
- real-run (shell, 2026-10-06): `ls .github/workflows/` (8 файлов:
  7 yml + README.md, review нет); `gh workflow list` (7 active — формат
  вывода для проверки регистрации); `python3 -c "import yaml"` (pyyaml
  доступен).
- real-run (чтение файлов, 2026-10-06): `.github/workflows/opencode.yml`
  — рабочая связка env/model task-8 и стиль шагов (консистентность
  репозитория); task-9 step-2 (формат разбора и отрицательных
  проверок); task-8 cycle report (путь B, perm-гейт).
- code-reading: pull_request-воркфлоу берётся из merge-ref PR — потому
  ci:-PR сам себе даёт первый прогон (`opened`) до мержа в `main`;
  403-риск постинга на read-правах — семантика разрешений GitHub
  (комментарий = write), проверяется живым прогоном; `share: false` —
  из подтверждённого наблюдения task-9 (триаж-комментарий без
  share-ссылки, самопробуждение ограничилось `skipped`).
