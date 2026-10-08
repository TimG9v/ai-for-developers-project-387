# step-1-variable-and-workflow-to-main

- Дата: 2026-10-06 18:35 (+05)
- Ветка: `task-11` (создаётся от свежего `main` в начале шага);
  мерж через PR, `--merge`
- Шаг: план шага 1/3 — переменная репозитория `APP_URL`, файл
  `opencode-scheduled.yml` (schedule + workflow_dispatch, Lighthouse,
  агент с prompt), PR → `main`, мерж, регистрация воркфлоу

Исходный план: `20261006183526-implementation-plan.md`.

## Цель

Переменная репозитория `APP_URL` = `https://calendar-387.onrender.com`
задана (`gh variable list` показывает её);
`.github/workflows/opencode-scheduled.yml` замержен в `main`: события
`schedule` (cron `0 6 * * *`, `timezone: Europe/Moscow`) и
`workflow_dispatch`; права `id-token`/`contents`/`pull-requests`/
`issues` — `write`; непустой `prompt` с инструкцией создать issue;
воркфлоу зарегистрирован `active`. Живой прогон — не в этом шаге
(dispatch только с дефолтной ветки, шаг 2).

## Interfaces

- Consumes: docs Schedule Example (структура файла, write-набор прав,
  обязательный prompt); связка model/env из действующего `opencode.yml`
  (`ZHIPU_API_KEY` + `zai-coding-plan/glm-5.3-flash`, task-8 путь B);
  прецедент app-токена на `issues: write` (`opencode-triage.yml`,
  task-9); публичный адрес из README; конвенция PR → `main`
  (task-7/8/9/10).
- Produces: воркфлоу `opencode-scheduled` в `main` + переменная
  `APP_URL` — вход шага 2 (ручной запуск, артефакт, issue) и всех
  ночных прогонов.

## Предусловия (real-run перед стартом)

- [ ] 1. Дерево чистое, `main` свежий, файла-расписания нет,
  переменной нет:

  ```bash
  git checkout main && git pull && git status --porcelain
  ls .github/workflows/ | grep -i sched
  gh variable list
  ```

  Ожидание: `status` пуст; `grep` ничего не нашёл (exit 1);
  `gh variable list` — пусто.

- [ ] 2. Приложение отвечает (иначе чинить деплой, а не расписание):

  ```bash
  curl -s -o /dev/null -w "%{http_code} %{time_total}s\n" \
    --max-time 90 https://calendar-387.onrender.com
  ```

  Ожидание: `200`; время может быть десятки секунд (cold start —
  учтён warm-up-шагом в воркфлоу).

## Действия

- [ ] 1. Создать переменную репозитория (настройка вне git, коммита
  не требует):

  ```bash
  gh variable set APP_URL --body "https://calendar-387.onrender.com"
  gh variable list
  ```

  Ожидание: в списке `APP_URL`. Адрес — публичный (README), поэтому
  переменная, не секрет.

- [ ] 2. Создать рабочую ветку:

  ```bash
  git checkout -b task-11
  ```

- [ ] 3. (По решению пользователя) `docs:`-коммит контекст-файлов
  плана:

  ```bash
  git add docs/context/tmp/task-11
  git commit -m "docs: add task-11 context docs"
  ```

  Образец — task-9 (`e263a7f`), task-10.

- [ ] 4. Создать `.github/workflows/opencode-scheduled.yml` целиком:

  ```yaml
  name: opencode-scheduled

  on:
    schedule:
      - cron: "0 6 * * *" # ежедневно 06:00 по Москве
        timezone: Europe/Moscow
    workflow_dispatch:

  jobs:
    quality-check:
      runs-on: ubuntu-latest
      permissions:
        id-token: write
        contents: write
        pull-requests: write
        issues: write
      steps:
        - name: Checkout repository
          uses: actions/checkout@v6
          with:
            fetch-depth: 1
            persist-credentials: false

        - name: Warm up app (Render cold start)
          run: |
            code="000"
            for i in $(seq 1 12); do
              code=$(curl -s -o /dev/null -w "%{http_code}" \
                --max-time 60 "${{ vars.APP_URL }}")
              echo "attempt $i: HTTP $code"
              [ "$code" = "200" ] && break
              sleep 10
            done
            [ "$code" = "200" ] || {
              echo "app did not return 200, last code: $code" >&2
              exit 1
            }

        - name: Collect Lighthouse report
          run: |
            mkdir -p lighthouse
            npx --yes lighthouse@13 "${{ vars.APP_URL }}" \
              --output json \
              --output-path ./lighthouse/report.json \
              --chrome-flags="--headless=new" \
              --quiet

        - name: Upload report artifact
          uses: actions/upload-artifact@v7
          with:
            name: lighthouse-report
            path: lighthouse/report.json

        - name: Today
          id: today
          run: echo "date=$(date +%F)" >> "$GITHUB_OUTPUT"

        - name: Run OpenCode
          uses: anomalyco/opencode/github@latest
          env:
            ZHIPU_API_KEY: ${{ secrets.ZHIPU_API_KEY }}
          with:
            model: zai-coding-plan/glm-5.3-flash
            share: false
            prompt: |
              Прочитай файл lighthouse/report.json — свежий отчёт
              Lighthouse публичной страницы приложения «Календарь
              звонков» (адрес в переменной репозитория APP_URL).
              Проанализируй оценки категорий (performance,
              accessibility, best-practices, seo) и метрики Core Web
              Vitals (FCP, LCP, TBT, CLS, SI).
              Затем создай РОВНО ОДНУ новую issue в этом репозитории:
              - заголовок: «Lighthouse: регулярная проверка
                ${{ steps.today.outputs.date }}»;
              - тело: 1) оценки четырёх категорий, 2) топ-5 проваленных
                аудитов с наибольшим весом и их метрики, 3) по каждой
                проблеме — конкретная рекомендация (что править в
                frontend/, если источник очевиден).
              Не создавай ветки, коммиты и pull request'ы. Не
              комментируй существующие issues. Итог прогона — только
              эта issue.
  ```

- [ ] 5. Разбор файла — осознанные решения относительно docs Schedule
  Example:

  - события: `schedule` + `workflow_dispatch` (docs-пример имеет
    только schedule; спека требует оба — ручной прогон без ожидания
    ночи);
  - `timezone: Europe/Moscow` рядом с `cron` — поддержка подтверждена
    docs workflow-syntax; cron остаётся пятиполевым `0 6 * * *`;
  - провайдер/модель/секрет: `zai-coding-plan/glm-5.3-flash` +
    `ZHIPU_API_KEY` вместо `anthropic/…` + `ANTHROPIC_API_KEY`
    (рабочая связка task-8);
  - режим OIDC/GitHub App — без `use_github_token` и `GITHUB_TOKEN`
    (docs Schedule Example; прецедент task-9: app-токен с
    `issues: write` работает);
  - `share: false` — комментарии/issue агента без ссылки на
    shared-сессию не самопробуждают `opencode.yml` (task-9);
  - warm-up-шаг — дополнение к спеке-примеру: холодный старт Render
    (real-run 23.2 s) иначе попадёт в метрики Lighthouse;
  - `lighthouse@13` — пин мажора (determinism), один json-артефакт;
  - `steps.today.outputs.date` в prompt — дата прогона в заголовке
    issue от агента не зависит (выражения в `with:` подставляются
    GitHub до запуска экшена).

- [ ] 6. Локальная проверка YAML:

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-scheduled.yml')); print('YAML-OK')"
  grep -n "cron:" .github/workflows/opencode-scheduled.yml
  grep -n "timezone:" .github/workflows/opencode-scheduled.yml
  grep -n "workflow_dispatch:" .github/workflows/opencode-scheduled.yml
  grep -n "vars.APP_URL" .github/workflows/opencode-scheduled.yml
  ```

  Ожидание: `YAML-OK`; по одному `cron:` и `timezone:` внутри
  schedule; `workflow_dispatch:` присутствует; адрес в файле — только
  через `vars.APP_URL`, литерала адреса нет.

- [ ] 7. Точечный коммит (только новый файл воркфлоу):

  ```bash
  git add .github/workflows/opencode-scheduled.yml
  git status --porcelain
  git commit -m "ci: add scheduled lighthouse quality check workflow"
  ```

  Ожидание: в индексе один файл (плюс docs-коммит, если взят);
  Conventional Commit `ci:`.

- [ ] 8. PR → `main`, дождаться CI, мерж:

  ```bash
  git push -u origin task-11
  gh pr create --title "ci: add scheduled lighthouse quality check workflow" --fill
  gh pr checks --watch
  gh pr merge --merge
  ```

  Ожидание: проверки зелёные. Внимание: этот PR — `pull_request`-ивент,
  наш воркфлоу на него не реагирует (его события — schedule и
  workflow_dispatch); сам `opencode-review` (task-10) оставит свои
  замечания — мерж не блокировать из-за замечаний к самому воркфлоу
  (они — материал докоммита при необходимости).

- [ ] 9. Регистрация воркфлоу:

  ```bash
  git fetch
  git show origin/main:.github/workflows/opencode-scheduled.yml >/dev/null && echo MAIN-OK
  gh workflow list | grep scheduled
  gh workflow view opencode-scheduled
  ```

  Ожидание: `MAIN-OK`; строка `opencode-scheduled … active`; в выводе
  `view` видны оба события — schedule (cron) и workflow_dispatch.
  Прогонов ещё нет — это нормально (запуск — шаг 2).

## Файлы

- Create: `.github/workflows/opencode-scheduled.yml`.
- Настройка репо (вне git): переменная `APP_URL`.
- Modify: ничего; предсуществующие воркфлоу (включая `opencode.yml`,
  `opencode-triage.yml`, `opencode-review.yml`, `release-please.yml`)
  не трогаются.
- Не трогать: код, контракт, README, `hexlet-check.yml`.

## Проверка (real-run)

| # | Команда                                                                   | Ожидаемый результат                                        |
| - | ------------------------------------------------------------------------- | ---------------------------------------------------------- |
| 1 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/opencode-scheduled.yml'))"` | без исключения                            |
| 2 | `git show origin/main:.github/workflows/opencode-scheduled.yml`           | файл в `main`, содержимое = действие 4                     |
| 3 | `gh pr view --json state -q .state` (по URL PR)                           | `MERGED`                                                   |
| 4 | `gh variable list`                                                        | `APP_URL` в списке                                         |
| 5 | `gh workflow list \| grep scheduled`                                      | `opencode-scheduled … active`                              |
| 6 | `gh workflow view opencode-scheduled`                                     | события schedule (cron 0 6 \* \* \*) и workflow_dispatch   |

## Отрицательные проверки

- В файле нет значения секрета и нет литерала адреса приложения:

  ```bash
  grep -n "secrets\." .github/workflows/opencode-scheduled.yml
  grep -n "onrender.com" .github/workflows/opencode-scheduled.yml
  ```

  Ожидание: первая команда находит только ссылку
  `${{ secrets.ZHIPU_API_KEY }}`; вторая — ничего (exit 1).

- События файла — только `schedule` и `workflow_dispatch`:

  ```bash
  grep -n -A 6 "^on:" .github/workflows/opencode-scheduled.yml
  ```

  Ожидание: без `issues:` / `pull_request:` — дублей триггеров с
  существующими opencode-воркфлоу нет.

- Cron-элемент один (требование «не чаще раза в сутки»):

  ```bash
  grep -c "cron:" .github/workflows/opencode-scheduled.yml
  ```

  Ожидание: `1`.

- Дифф PR против `main` — только новый файл (+ `docs:`-коммит
  контекстов, если взят):

  ```bash
  git diff origin/main...HEAD --name-only
  ```

## Коммит

Один коммит: `ci: add scheduled lighthouse quality check workflow`
(файл `.github/workflows/opencode-scheduled.yml`); опционально
предшествующий `docs:`-коммит контекст-файлов. В `main` — через PR,
мерж `--merge`. Переменная `APP_URL` — настройка репо, в коммит не
попадает.

## Результат шага (description.md)

В `.github/workflows/` есть воркфлоу с событиями `schedule` (cron
задан, с учётом часового пояса) и `workflow_dispatch`; у воркфлоу
задан prompt и права `contents: write`, `pull-requests: write`,
`issues: write` вместе с `id-token: write`; частота — раз в сутки;
адрес проверки — переменная репозитория. Живой прогон и его
артефакты — шаг 2.

## Источники

- real-run (fetch, 2026-10-06): docs.github.com workflow syntax —
  `on.schedule` (cron — POSIX, пять полей; UTC по умолчанию; `timezone`
  — IANA-строка рядом с `cron`; пример `- cron: '30 5 * * 1-5'` +
  `timezone: "America/New_York"`; минимальный интервал 5 минут;
  расписание берёт последний коммит дефолтной ветки),
  `on.workflow_dispatch` («trigger only receives events when the
  workflow file is on the default branch» — ручной запуск только после
  мержа); opencode.ai/docs/github — Schedule Example (структура файла:
  `id-token`/`contents`/`pull-requests`/`issues` write, checkout с
  `persist-credentials: false`, prompt обязателен; «Output goes to
  logs and PRs»), Configuration (`model` формат provider/model,
  `share` default true для публичных репо, `prompt` override).
- real-run (shell, 2026-10-06): `ls .github/workflows/` (8 файлов,
  scheduled нет); `gh workflow list` (формат вывода для проверки
  регистрации); `gh variable list` (пусто — переменную создаём);
  `npm view lighthouse version` (13.5.0 → пин @13); `gh api repos/
  actions/upload-artifact/releases/latest` (v7.0.1 → пин @v7); `curl`
  публичного URL (HTTP 200 за 23.2 s — cold start реален); `python3
  -c "import yaml"` (pyyaml доступен).
- real-run (чтение файлов, 2026-10-06): `.github/workflows/opencode.yml`
  (связка env/model task-8, стиль шагов — консистентность репозитория),
  `opencode-triage.yml` (прецедент prompt-режима с write-правами на
  issues и `share: false`), `opencode-review.yml` (task-10, стиль
  разбора файла), `README.md` (публичный URL, cold start); task-9
  step-2 (формат отрицательных проверок).
- code-reading: warm-up-шаг — следствие cold-start Render (README +
  замер 23.2 s) и природы Lighthouse (первая страница = метрики);
  подстановка `${{ steps.today.outputs.date }}` внутри блока
  `prompt:` — выражения GitHub вычисляются в `with:` до передачи
  значения экшену; agent-созданная issue триггерит `opencode-triage`
  (`issues: opened`) — ожидаемый побочный конвейер, зацикливания нет
  (task-9); переменная вместо секрета — адрес публичный, секреты для
  значений, требующих сокрытия (спека прямо указывает Variables).
