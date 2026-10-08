# step-1-workflow-to-main

- Дата: 2026-10-05 15:49
- Ветка: task-8 (уже создана от `main` @ `2dbe5d9`; PR этой ветки — в `main`)
- Шаг: план шага 1/3 — воркфлоу агента в основную ветку + разбор воркфлоу

Исходный план: `20261005154949-implementation-plan.md`.

## Цель

Файл `.github/workflows/opencode.yml` замержен в `main` через PR, локально
валиден как YAML, предсуществующие воркфлоу на `main` зелёные; разбор
воркфлоу (события, условие, права, модель) зафиксирован в этом файле как
артефакт задачи 3 описания.

## Interfaces

- Consumes: ветка `task-8` от `main` @ `2dbe5d9` (создана заранее).
- Produces: merge-коммит PR в `main` с `.github/workflows/opencode.yml`;
  зарегистрированный в репозитории воркфлоу `opencode` (виден в
  `gh workflow list`); секрет `OPENCODE_API_KEY` — имя, на которое шаг 2
  ссылается при создании; модель `opencode/glm-5.3-flash` — значение,
  которое шаг 3 проверяет в логах прогона.

## Предусловия (real-run перед стартом)

- [ ] 1. Ветка и актуальность:

  ```bash
  git branch --show-current && git fetch && git status --porcelain
  ```

  Ожидание: `task-8`; `main` не отстаёт от `origin/main`; в выводе status —
  только пользовательская правка `docs/context/task-7pre/description.md`
  (real-run 2026-10-05), она в коммиты шага не входит.

- [ ] 2. Файл воркфлоу ещё не существует:

  ```bash
  ls .github/workflows/opencode.yml
  ```

  Ожидание: «No such file or directory».

## Действия

- [ ] 1. Создать `.github/workflows/opencode.yml` (по docs/github manual
  setup, real-run fetch 2026-10-05; замены — env `OPENCODE_API_KEY` и
  модель `opencode/…`):

  ```yaml
  name: opencode

  on:
    issue_comment:
      types: [created]
    pull_request_review_comment:
      types: [created]

  jobs:
    opencode:
      if: |-
        contains(github.event.comment.body, '/oc') ||
        contains(github.event.comment.body, '/opencode')
      runs-on: ubuntu-latest
      permissions:
        id-token: write
      steps:
        - name: Checkout repository
          uses: actions/checkout@v6
          with:
            fetch-depth: 1
            persist-credentials: false

        - name: Run OpenCode
          uses: anomalyco/opencode/github@latest
          env:
            OPENCODE_API_KEY: ${{ secrets.OPENCODE_API_KEY }}
          with:
            model: opencode/glm-5.3-flash
            # share: true
  ```

- [ ] 2. Локальная валидация YAML (actionlint/yamllint не установлены,
  real-run 2026-10-05; GitHub провалидирует остальное на push):

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode.yml'))" && echo YAML-OK
  ```

  Ожидание: `YAML-OK`, exit 0.

- [ ] 3. Проверить структуру обязательных требований спеки по файлу
  (real-run grep):

  ```bash
  grep -n "types: \[created\]\|id-token: write\|persist-credentials: false\|contains(github.event.comment.body" .github/workflows/opencode.yml
  ```

  Ожидание: `types: [created]` — два вхождения (оба события);
  `id-token: write`, `persist-credentials: false`, условие `contains` —
  по одному.

- [ ] 4. Коммит (только файл воркфлоу; правка
  `docs/context/task-7pre/description.md` не входит):

  ```bash
  git add .github/workflows/opencode.yml
  git commit -m "ci: add opencode agent workflow for issue and pr comments"
  git status --porcelain
  ```

  Ожидание: в статусе остаётся только ` M docs/context/task-7pre/description.md`.

- [ ] 5. Push, PR, мерж (конвенция task-7: merge-коммит):

  ```bash
  git push -u origin task-8
  gh pr create --base main --head task-8 \
    --title "ci: add opencode agent workflow" \
    --body "Воркфлоу агента OpenCode: issue_comment и
  pull_request_review_comment, types [created], условие /oc|/opencode,
  права id-token: write, checkout без persist-credentials, ключ в
  OPENCODE_API_KEY. Спека: docs/context/tmp/task-8/description.md"
  gh pr merge --merge --delete-branch=false
  ```

  Ожидание: PR создан; merge-коммит в `main`; CI на PR зелёное
  (backend/frontend/hexlet-check/release-please — воркфлоу агента на PR
  не запускается: нет события-комментария).

- [ ] 6. Воркфлоу зарегистрирован в репозитории:

  ```bash
  gh workflow list
  ```

  Ожидание: в списке есть `opencode` (state `active`), рядом с пятью
  предсуществующими.

## Разбор воркфлоу (артефакт задачи 3 описания)

| Вопрос                       | Ответ по файлу                                                                                                                               |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Какие события запускают      | `issue_comment` (issue или PR-дискуссия) и `pull_request_review_comment` (строки diff), оба `types: [created]` — правка комментария не будит |
| Что пропускает работу дальше | `if` на job: тело комментария содержит `/oc` или `/opencode`; стартует на каждый комментарий, работа — только при команде                    |
| Какие права выданы           | только `id-token: write`: OIDC подтверждает запуск из репо, дальше — права GitHub App; runner-токен не нужен                                 |
| Какая модель указана         | `opencode/glm-5.3-flash` (OpenCode Zen); ключ — секрет `OPENCODE_API_KEY`                                                                    |
| Почему checkout без прав     | токен раннера агенту не нужен: аутентификация через OIDC-обмен против установленного App                                                     |

## Файлы

- Create: `.github/workflows/opencode.yml`.
- Modify: ничего.
- Не трогать: остальные `.github/workflows/**` (включая README.md и
  `hexlet-check.yml`), `backend/**`, `frontend/**`, `contracts/**`,
  `README.md`, `AGENTS.md`, `docs/**`.

## Проверка (real-run)

| # | Команда                                                                            | Ожидаемый результат                                                              |
| - | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| 1 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode.yml'))"` | exit 0                                                                           |
| 2 | `gh pr view --json state,mergeCommit` (по номеру PR)                               | `MERGED`, sha не пустой                                                          |
| 3 | `gh workflow list`                                                                 | `opencode` в списке                                                              |
| 4 | `gh run list --branch main --limit 6`                                              | backend-ci, frontend-ci, hexlet-check, release-please — success на merge-коммите |
| 5 | `git cat-file -e main:.github/workflows/opencode.yml && echo FILE-IN-MAIN`         | `FILE-IN-MAIN`, exit 0                                                           |
| 6 | `git status --porcelain`                                                           | только пользовательская правка description.md                                    |

## Коммит

Выполняет агент в ветке `task-8`:
`ci: add opencode agent workflow for issue and pr comments`.
Один шаг — один коммит; в `main` попадает через merge PR.

## Предусловия следующих шагов

Шаг 2 требует от шага 1: воркфлоу в `main` (имя секрета
`OPENCODE_API_KEY` уже зафиксировано в файле), зелёный CI. Шаг 3 требует
дополнительно от шага 2: установленный App и существующий секрет.

## Источники

- real-run (fetch, 2026-10-05): opencode.ai/docs/github — YAML manual
  setup (эталон файла), параметры `model`/`mentions`/`share`.
- real-run (shell, 2026-10-05): ветка `task-8` @ `2dbe5d9`, status (одна
  пользовательская правка), `ls .github/workflows/` (opencode.yml нет),
  `which actionlint yamllint` (нет), `python3 -c "import yaml"` (pyyaml
  есть), `gh run list` (CI success на `main`).
- code-reading: семантика `issue_comment` (воркфлоу исполняется из
  дефолтной ветки — потому PR→merge до живого вызова) — устройство
  GitHub Actions; тип коммита `ci:` — Conventional Commits.
