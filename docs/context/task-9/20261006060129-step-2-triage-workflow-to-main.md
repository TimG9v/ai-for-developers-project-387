# step-2-triage-workflow-to-main

- Дата: 2026-10-06 06:01 (+05)
- Ветка: `task-9` (создаётся от свежего `main` в начале шага);
  мерж через PR
- Шаг: план шага 2/3 — файл воркфлоу автотриажа на событии `issues`
  с заданным `prompt`, PR → `main`, мерж, разбор

Исходный план: `20261006060129-implementation-plan.md`.

## Цель

`.github/workflows/opencode-triage.yml` существует в `main` (merge
PR): событие `issues` c `types: [opened]`, непустой `prompt`, права
`id-token: write` + `issues: write`, checkout без persist-credentials;
воркфлоу зарегистрирован `active`; CI на `main` зелёный.

## Interfaces

- Consumes: связка env/model из действующего `opencode.yml`
  (`ZHIPU_API_KEY` + `zai-coding-plan/glm-5.3-flash`, task-8 путь B);
  Issues Triage Example из docs opencode (структура файла); конвенция
  PR → `main` (task-7/8).
- Produces: воркфлоу автотриажа в `main` — единственный вход шага 3
  (живая проверка на новой задаче; событийный воркфлоу берётся из
  дефолтной ветки).

## Предусловия (real-run перед стартом)

- [ ] 1. Дерево чистое, `main` свежий, файла-триажа нет:

  ```bash
  git checkout main && git pull && git status --porcelain
  ls .github/workflows/ | grep triage
  ```

  Ожидание: `status` пуст; `grep` ничего не нашёл (exit 1).

## Действия

- [ ] 1. Создать рабочую ветку:

  ```bash
  git checkout -b task-9
  ```

- [ ] 2. Создать `.github/workflows/opencode-triage.yml` целиком:

  ```yaml
  name: opencode-triage

  on:
    issues:
      types: [opened]

  jobs:
    triage:
      runs-on: ubuntu-latest
      permissions:
        id-token: write
        issues: write
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
          with:
            model: zai-coding-plan/glm-5.3-flash
            share: false
            prompt: |
              Затриажь эту issue как участник команды календарного приложения.
              Оставь один комментарий с разбором:
              1. Суть проблемы своими словами: что сломано и у кого.
              2. Причина в коде: какие файлы и модули отвечают за описанное
                 поведение и почему оно такое (проверь backend/ и frontend/).
              3. Предлагаемый путь исправления по шагам.
              Если данных в issue не хватает, задай уточняющие вопросы в том
              же комментарии. Если сообщение не про приложение, объясни это
              и предложи закрыть issue.
  ```

- [ ] 3. Разбор файла (задача «воркфлоу с заданным prompt») — пять
  осознанных отклонений от docs-примера `opencode-triage.yml`:

  - провайдер/модель/секрет: `zai-coding-plan/glm-5.3-flash` +
    `ZHIPU_API_KEY` вместо `anthropic/…` + `ANTHROPIC_API_KEY`
    (инцидент task-8: Zen-ключ vs coding-plan);
  - без гейта возраста аккаунта: единственный автор issue — владелец,
    гейт всегда `true` (анти-спам внешних авторов тут неприменим);
  - права `id-token: write` + `issues: write` вместо четырёх из
    примера: триаж только комментирует, ветки/PR не создаёт
    (docs: contents/pull-requests write — под создание веток/PR);
  - `share: false`: ответ без ссылки на shared-сессию не содержит
    `//opencode` и не самопробуждает коммент-воркфлоу (гипотеза из
    наблюдений task-8, проверка — шаг 3);
  - `prompt` на русском с гарантией комментария и критериями
    (суть/причина+файлы/путь) — зеркалит ручной разбор шага 1 для
    честного сравнения; docs-овское «Otherwise, do not comment»
    убрано, иначе живое доказательство не фиксируется комментарием.

- [ ] 4. Локальная проверка YAML:

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-triage.yml')); print('YAML-OK')"
  grep -n "types: \[opened\]" .github/workflows/opencode-triage.yml
  grep -n "prompt:" .github/workflows/opencode-triage.yml
  ```

  Ожидание: `YAML-OK`; обе grep-строки найдены (review focus 1 и 3).

- [ ] 5. Точечный коммит (только новый файл):

  ```bash
  git add .github/workflows/opencode-triage.yml
  git status --porcelain
  git commit -m "ci: add opencode auto-triage workflow for new issues"
  ```

  Ожидание: в индексе один файл; Conventional Commit `ci:`.

- [ ] 6. PR → `main`, дождаться CI, мерж:

  ```bash
  git push -u origin task-9
  gh pr create --title "ci: add opencode auto-triage workflow for new issues" --fill
  gh pr checks --watch
  gh pr merge --merge
  ```

  Ожидание: проверки зелёные; merge-коммит в истории `main`.

- [ ] 7. Регистрация воркфлоу и наличие файла в `main`:

  ```bash
  git fetch
  git show origin/main:.github/workflows/opencode-triage.yml >/dev/null && echo MAIN-OK
  gh workflow list | grep -i triage
  ```

  Ожидание: `MAIN-OK`; строка `opencode-triage … active` (после
  мержа регистрация может занять секунды — повторить просмотр).

## Файлы

- Create: `.github/workflows/opencode-triage.yml`.
- Modify: ничего; предсуществующие воркфлоу (включая `opencode.yml`)
  не трогаются.
- Не трогать: код, контракт, README, `.github/workflows/README.md`.

## Проверка (real-run)

| # | Команда                                                                                   | Ожидаемый результат                    |
| - | ----------------------------------------------------------------------------------------- | -------------------------------------- |
| 1 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-triage.yml'))"` | без исключения                         |
| 2 | `git show origin/main:.github/workflows/opencode-triage.yml`                              | файл в `main`, содержимое = шаг 2      |
| 3 | `gh pr view --json state -q .state` (по URL PR)                                           | `MERGED`                               |
| 4 | `gh workflow list \| grep -i triage`                                                      | `opencode-triage … active`             |
| 5 | `gh run list --branch main --limit 4`                                                     | предсуществующие workflows — `success` |

## Отрицательные проверки

- В файле нет значений секретов — только ссылка
  `${{ secrets.ZHIPU_API_KEY }}` (`grep -c "ZHIPU" файл` = 2:
  env-строка и ссылка; значение ключа в файле запрещено).
- События файла — только `issues` (`grep -n "on:" -A 3`): ни
  `issue_comment`, ни `pull_request` — дублирования запусков с
  существующим воркфлоу нет.

## Коммит

Один коммит: `ci: add opencode auto-triage workflow for new issues`
(файл `.github/workflows/opencode-triage.yml`); вливается в `main`
через PR. Контекст-файлы — отдельным `docs:`-коммитом по решению
пользователя.

## Результат шага (description.md)

В `.github/workflows/` есть воркфлоу автотриажа на событии `issues`
с заданным prompt; живое подтверждение срабатывания на новой задаче —
шаг 3.

## Источники

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — Issues
  Triage Example (структура, `on.issues.types`, обязательный
  `prompt`, права примера), Supported Events (для `issues` prompt
  требуется), Configuration (`share`, `model`, `mentions`).
- real-run (shell, 2026-10-06): `ls .github/workflows/` (opencode-
  triage.yml отсутствует); `gh workflow list` (6 active — формат
  вывода для проверки регистрации); `python3 -c "import yaml"`
  (pyyaml доступен; actionlint/yamllint не установлены).
- real-run (чтение файлов, 2026-10-06): `.github/workflows/
  opencode.yml` — рабочая связка env/model task-8 (копируется
  без изменений); docs/context/task-8/20261006054241-cycle-report.md
  (инцидент AuthError, perm-гейт, самопробуждение).
- code-reading: права `issues: write` — из назначения триажа
  (комментарий в issue); `fetch-depth: 1` и стиль шагов — из
  существующего `opencode.yml` (консистентность репозитория);
  отсутствие yamllint в проекте — lint файла ограничен парсингом.
