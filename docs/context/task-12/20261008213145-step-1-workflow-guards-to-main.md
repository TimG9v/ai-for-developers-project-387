# step-1-workflow-guards-to-main

- Дата: 2026-10-08 21:31 (+05)
- Ветка: `task-12` — существует локально, отстаёт от `main`; на старте
  шага fast-forward `git merge --ff-only origin/main` (уникальных
  коммитов нет); мерж через PR, `--merge`
- Шаг: план шага 1/3 — бот-фильтр в `opencode-triage.yml`,
  `mentions: /oc` + `share: false` + ограничение вызывающих в
  `opencode.yml`, понижение `issues: write` → `read` в
  `opencode-review.yml`; PR `ci:` → `main`, мерж

Исходный план: `20261008213145-implementation-plan.md`.

## Цель

Три файла воркфлоу замержены в `main` одним `ci:`-PR: у триажа есть
бот-фильтр по автору issue; у интерактивного воркфлоу команда вызова
одна (`mentions: /oc`, строка `/opencode` из `if` удалена), круг
вызывающих ограничен `author_association`, `share: false` задан явно;
у ревью-воркфлоу `issues` понижен до `read`. Все воркфлоу после мержа
`active`. Живые прогоны — не в этом шаге (шаг 3).

## Interfaces

- Consumes: действующие `if`-условия и права трёх воркфлоу в `main`
  (`d54aae7`); параметры экшена `mentions`/`share` (docs Configuration);
  инцидент триажа #26 / прогон 37757599684 как обоснование бот-фильтра;
  конвенция PR → `main` (task-7–11).
- Produces: обновлённые `opencode.yml`, `opencode-triage.yml`,
  `opencode-review.yml` в `main` — вход шага 2 (README описывает ровно
  это состояние) и шага 3 (живые проверки условий).

## Предусловия (real-run перед стартом)

- [ ] 1. Ветка `task-12` без уникальных коммитов, fast-forward на
  свежий `main`:

  ```bash
  git checkout task-12
  git fetch
  git log --oneline origin/main..task-12   # пусто — уникальных нет
  git merge --ff-only origin/main
  git status --porcelain
  ```

  Ожидание: первый лог пуст; merge проходит без мерж-коммита; статус
  чистый. Если появились уникальные коммиты — остановиться и
  согласовать перебазирование с пользователем.

- [ ] 2. Файлы в состоянии `main` (бот-фильтр triage отсутствует,
  mentions отсутствует, review имеет `issues: write`):

  ```bash
  grep -c "user.type != 'Bot'" .github/workflows/opencode-triage.yml
  grep -c "mentions:" .github/workflows/opencode.yml
  grep -n "issues:" .github/workflows/opencode-review.yml
  ```

  Ожидание: `0` (exit 1), `0` (exit 1), строка `issues: write`.

## Действия

- [ ] 1. `opencode-triage.yml` — добавить `if` в job `triage`
  (перед `runs-on`):

  ```yaml
  jobs:
    triage:
      # Issue от бота (opencode-agent[bot] после Lighthouse-прогона)
      # стартовала триаж, который падал на внутреннем perm-гейте экшена:
      # у bot-автора нет collaborator-прав, агент не стартует, бот
      # получает мусорный комментарий с ошибкой (issue #26, прогон
      # 37757599684). Ботов отсекаем до старта job — нулевой расход.
      if: github.event.issue.user.type != 'Bot'
      runs-on: ubuntu-latest
  ```

- [ ] 2. `opencode.yml` — заменить `if` job'а (старый: два `contains`
  + бот-фильтр; комментарий над условием сохранить и дополнить):

  ```yaml
  jobs:
    opencode:
      # Бот-фильтр: ответ агента сам является комментарием — без отсечения
      # ботов воркфлоу перезапускается сам собой (петля, task-8). Дальше
      # permission-гейт CLI проверяет actor'а, у бота collaborator-permission
      # = none, прогон падает с мусорным комментарием.
      #
      # Круг вызывающих: в публичном репозитории комментировать может
      # кто угодно; агент для посторонних не должен даже стартовать.
      # OWNER/COLLABORATOR/MEMBER проходят, остальные — skipped (нулевой
      # расход минут и токенов). Вторым слоем остаётся perm-гейт CLI.
      if: |-
        contains(github.event.comment.body, '/oc') &&
        github.event.comment.user.type != 'Bot' &&
        (github.event.comment.author_association == 'OWNER' ||
        github.event.comment.author_association == 'COLLABORATOR' ||
        github.event.comment.author_association == 'MEMBER')
      runs-on: ubuntu-latest
  ```

- [ ] 3. `opencode.yml` — в шаге `Run OpenCode` заменить строку
  `# share: true` на явные параметры:

  ```yaml
        with:
          model: zai-coding-plan/glm-5.3-flash
          # Одна команда вызова вместо дефолтной пары /opencode,/oc:
          # вся практика task-8..11 — /oc; подстрока /opencode встречается
          # в ссылках opencode.ai и запускала воркфлоу на комментарии
          # бота (самопробуждение task-8).
          mentions: /oc
          # Публичный репозиторий: дефолт share=true публикует сессию
          # с полным контекстом по ссылке — решение закрыто явно (task-12).
          share: false
  ```

- [ ] 4. `opencode-review.yml` — понизить `issues: write` до `read`:

  ```yaml
    permissions:
      id-token: write
      contents: read
      pull-requests: write
      # Ревью пишет только комментарий к PR; issues читает (связанная
      # задача), но не изменяет — write понижен до read (task-12).
      issues: read
  ```

- [ ] 5. Разбор правок — осознанные решения:

  - бот-фильтр triage — по автору issue (`github.event.issue.user.type`),
    не по комментарию: событие `issues: opened`;
  - `mentions` выровнен с `if`: обе проверки теперь знают только
    `/oc`; экшен сам парсит команду после mention;
  - `author_association` вычисляет GitHub до старта job — посторонний
    комментарий даёт `skipped`, агент и модель не запускаются;
  - `share: false` — строкой, не комментарием: закомментированный
    `# share: true` и был причиной публичных сессий (дефолт для
    публичных репозиториев);
  - `issues: read` в review — при явном permissions-блоке незаданные
    скоупы стали бы `none`, поэтому read задан явно (чтение связанных
    issue сохраняется);
  - `opencode-scheduled.yml` не трогается: write-набор оправдан docs
    Schedule Example и сайд-каналом PR #27, у schedule/dispatch нет
    автора события для фильтра.

- [ ] 6. Локальная проверка YAML и целевых строк:

  ```bash
  for f in opencode opencode-triage opencode-review; do
    python3 -c "import yaml; yaml.safe_load(open('.github/workflows/$f.yml')); print('$f YAML-OK')"
  done
  grep -n "mentions:" .github/workflows/opencode.yml
  grep -n "share:" .github/workflows/opencode.yml
  grep -n "author_association" .github/workflows/opencode.yml
  grep -n "user.type != 'Bot'" .github/workflows/opencode-triage.yml
  grep -n "issues:" .github/workflows/opencode-review.yml
  ```

  Ожидание: три `YAML-OK`; `mentions: /oc`; `share: false`;
  три ветки author_association; `if` в triage; `issues: read`.

- [ ] 7. Точечный коммит (только три файла воркфлоу):

  ```bash
  git add .github/workflows/opencode.yml .github/workflows/opencode-triage.yml .github/workflows/opencode-review.yml
  git status --porcelain
  git commit -m "ci: укрепить гаранты агентных воркфлоу — боты, mentions, share, права"
  ```

  Ожидание: в индексе ровно три файла; Conventional Commit `ci:`.

- [ ] 8. PR → `main`, дождаться проверок, мерж:

  ```bash
  git push -u origin task-12
  gh pr create --title "ci: укрепить гаранты агентных воркфлоу" --fill
  gh pr checks --watch
  gh pr merge --merge
  ```

  Ожидание: проверки зелёные (замечания `opencode-review` к самому
  PR — материал докоммита, мерж не блокировать; прецедент task-11
  step-1). Если required-проверка `build` красная из-за внешнего
  сбоя hexlet-check (прецедент 522, task-11 step-2) — `gh pr merge
  --admin --merge` по решению пользователя.

- [ ] 9. Регистрация и содержимое в `main`:

  ```bash
  git fetch
  for f in opencode opencode-triage opencode-review; do
    git show origin/main:.github/workflows/$f.yml >/dev/null && echo "$f MAIN-OK"
  done
  gh workflow list | grep opencode
  git diff origin/main...HEAD --name-only
  ```

  Ожидание: три `MAIN-OK`; все opencode-воркфлоу `active`; дифф PR —
  ровно три файла.

## Файлы

- Modify: `.github/workflows/opencode.yml` (`if`, `mentions`,
  `share`), `.github/workflows/opencode-triage.yml` (`if`),
  `.github/workflows/opencode-review.yml` (`permissions.issues`).
- Не трогать: `opencode-scheduled.yml`, `hexlet-check.yml`,
  CI-воркфлоу, `opencode.json`, код, README (README — шаг 2).

## Проверка (real-run)

| # | Команда                                                                                  | Ожидаемый результат                 |
| - | ---------------------------------------------------------------------------------------- | ----------------------------------- |
| 1 | `python3 -c "import yaml; …"` (три файла)                                                | без исключения                      |
| 2 | `git show origin/main:.github/workflows/opencode-triage.yml \| grep -c "Bot"`            | ≥ 1 (`if` в `main`)                 |
| 3 | `git show origin/main:.github/workflows/opencode.yml \| grep -n "mentions"`              | `mentions: /oc`                     |
| 4 | `git show origin/main:.github/workflows/opencode.yml \| grep -n "share"`                 | `share: false` (не комментарий)     |
| 5 | `git show origin/main:.github/workflows/opencode-review.yml \| grep -A 5 "permissions:"` | `issues: read`, write на issues нет |
| 6 | `gh pr view --json state -q .state` (URL PR)                                             | `MERGED`                            |
| 7 | `gh workflow list \| grep opencode`                                                      | 4 воркфлоу `active`                 |

## Отрицательные проверки

- В `opencode.yml` не осталось строки `'/opencode'` в `if`:

  ```bash
  grep -n "opencode'" .github/workflows/opencode.yml
  grep -n "contains(github.event.comment.body, '/opencode')" .github/workflows/opencode.yml
  ```

  Ожидание: второй grep пуст (exit 1); первый может находить только
  упоминания в комментариях.

- Закомментированного `# share: true` нет:

  ```bash
  grep -n "# share" .github/workflows/opencode.yml
  ```

  Ожидание: пусто (exit 1).

- В `opencode-review.yml` нет `issues: write`:

  ```bash
  grep -n "issues: write" .github/workflows/opencode-review.yml
  ```

  Ожидание: пусто (exit 1).

- Дифф PR против `main` — ровно три файла:

  ```bash
  git diff origin/main...HEAD --name-only
  ```

- Значения секретов в диффе отсутствуют (grep по диффу показывает
  только изменённые строки; ссылка на секрет в шаге `Run OpenCode`
  не менялась — в дифф её быть не должно):

  ```bash
  git diff origin/main...HEAD | grep -E "ZHIPU_API_KEY|RENDER_API_KEY"
  ```

  Ожидание: пусто (exit 1).

## Коммит

Один коммит: `ci: укрепить гаранты агентных воркфлоу — боты, mentions,
share, права` (три файла). В `main` — через PR, мерж `--merge`.
Опционально предшествующий `docs:`-коммит контекст-файлов плана
(образец task-9/10/11, по решению пользователя).

## Результат шага (description.md)

Бот-фильтр стоит во всех событийных агентных воркфлоу (triage — новый,
opencode/review — сохранены), набор команд задан явно (`mentions: /oc`),
лишний write в review убран, `share` закрыт явно (`false`), круг
вызывающих ограничен условием в воркфлоу. Записи в README и живые
проверки — шаги 2 и 3.

## Источники

- real-run (fetch, 2026-10-08): opencode.ai/docs/github — Configuration
  (`mentions` default `/opencode,/oc`; `share` default true для
  публичных; формат provider/model), Pull Request Example (read-набор
  для ревью), Schedule Example (write-набор обязателен для schedule);
  docs.github.com — workflow syntax (`if` вычисляется до старта job,
  skipped не расходует минуты; permissions: незаданные скоупы = none
  при явном блоке), event payload (`comment.user.type`,
  `comment.author_association`, `issue.user.type`).
- real-run (shell/чтение, 2026-10-08): состояние трёх файлов в `main`
  (бот-фильтры opencode/review есть, у triage нет; mentions нет;
  share закомментирован; issues: write в review); инцидент триажа #26
  (прогон 37757599684, failure + мусорный комментарий) — step-2-report
  task-11; самопробуждение на `/opencode` и perm-гейт для ботов —
  cycle-report task-8; `git diff origin/main HEAD` (ветка task-12
  отстаёт, triage/review идентичны main).
- code-reading: `author_association` OWNER/COLLABORATOR/MEMBER покрывает
  всех, кто может мержить, — посторонние имеют NONE (docs
  author_association values); сужение `if` до `/oc` не ломает ревью
  бот-PR (команду пишет человек — автор комментария человек); понижение
  до `read` не влияет на PR-комментарии (для них `pull-requests: write`).
