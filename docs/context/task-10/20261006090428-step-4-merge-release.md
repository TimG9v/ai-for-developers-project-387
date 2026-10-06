# step-4-merge-release

- Дата: 2026-10-06 09:04 (+05)
- Ветка: без рабочей ветки плана — мерж существующего PR агента через
  `gh`; release-PR создаёт release-please автоматически
- Шаг: план шага 4/4 — формат коммитов агента, мерж feature-PR,
  release-PR от release-please, приёмка цикла

Исходный план: `20261006090428-implementation-plan.md`.

## Цель

Коммиты агента проверены на Conventional Commits, feature-PR смержен
в `main` (`--merge`), Release Please собрал release-PR
(`chore(main): release 0.2.0`) с changelog из feat-коммитов агента;
приёмочная таблица всех требований заполнена (материал cycle-report).

## Interfaces

- Consumes: обновлённый PR агента (шаг 3, оба замечания учтены);
  `release-please.yml` (push → `main`,
  `googleapis/release-please-action@v4`, manifest `0.1.0`, config
  `simple`); конвенция мержа `--merge` (task-7/8/9).
- Produces: release-PR — артефакт приёмки (НЕ мержить: за пределами
  шага); материал cycle-report (номера PR/прогонов, хронология).

## Предусловия (real-run перед стартом)

- [ ] 1. PR агента готов: оба замечания учтены (шаг 3), CI зелёный:

  ```bash
  gh pr checks <N>
  ```

  Ожидание: все зелёные.

- [ ] 2. `main` свежий, дерево чистое:

  ```bash
  git checkout main && git pull && git status --porcelain
  ```

  Ожидание: `status` пуст.

## Действия

- [ ] 1. Проверка формата коммитов агента (требование 5, задача 6
  описания; review focus 4 — до мержа, не после):

  ```bash
  gh pr view <N> --json commits -q '.commits[].messageHeadline'
  ```

  Ожидание: каждая строка вида `type: subject` (feat|fix|ci|docs|
  chore|refactor|test|perf), минимум один `feat:`. Если коммит агента
  не по формату — вернуться в PR общим `/oc`-комментарием с просьбой
  переформулировать сообщения (шаг 3, цикл повторить) — после мержа
  править историю поздно.

- [ ] 2. Мерж (merge-коммит сохраняет индивидуальные коммиты агента —
  именно их разбирает release-please):

  ```bash
  gh pr merge <N> --merge
  ```

  Ожидание: merge-коммит в `main`; issue #7 закроется автоматически
  (`Closes #7` в теле PR).

- [ ] 3. Release Please на push → `main`; release-PR появился:

  ```bash
  gh run list --workflow "Release Please" --limit 2
  gh pr list --state open
  ```

  Ожидание: прогон `success`; открытый PR `chore(main): release 0.2.0`
  из ветки `release-please--branches--main` (feat → minor от `0.1.0`;
  фактическую версию читать из PR, ожидание `0.2.0` может сдвинуться
  при BREAKING CHANGE).

- [ ] 4. Состав release-PR и связь changelog ↔ коммиты агента:

  ```bash
  gh pr view <R> --json files -q '.files[].path'
  gh pr view <R> --json body | grep -A 5 "0.2.0"
  ```

  Ожидание: в файлах CHANGELOG.md и манифест/версия (состав — по
  факту PR; release-please simple без доп-конфига обновляет changelog
  и manifest); записи `0.2.0` содержат headline'ы feat-коммитов из PR
  агента — release-please разобрал именно коммиты агента (задача 6,
  требование 5).

- [ ] 5. Отрицательная механика (наблюдение, не дефект): release-PR
  создан `GITHUB_TOKEN` → pull_request-воркфлоу при его открытии не
  запускаются (документированное поведение GitHub Actions):

  ```bash
  gh run list --workflow opencode-review --limit 5
  ```

  Ожидание: нет прогона event `opened` по release-PR (только
  synchronize-прогоны feature-PR из шага 3).

- [ ] 6. (Опционально, решение пользователя — полное покрытие задачи 5
  «замечания на новом pull request»): закрыть и переоткрыть release-PR
  → событие `reopened` (в списке типов воркфлоу) → прогон
  `opencode-review` → замечания/резюме на release-PR:

  ```bash
  gh pr close <R> && gh pr reopen <R>
  gh run list --workflow opencode-review --limit 2
  ```

  Ожидание: прогон event `reopened`; комментарий авторевью в release-PR.
  Release-PR после reopen остаётся валидным — release-please обновит
  его при следующем push в `main`.

- [ ] 7. Приёмка цикла (все строки real-run; материал cycle-report):

| # | Требование (description.md)                             | Доказательство                       |
| - | ------------------------------------------------------- | ------------------------------------ |
| 1 | PR создан агентом, связан с issue                       | шаг 1, проверки 1–5                  |
| 2 | Оба вида замечаний (общий + к строке)                   | шаг 3, действия 1 и 3                |
| 3 | Правки в ту же ветку, видно по коммитам                 | шаг 3, проверки 1–2                  |
| 4 | Воркфлоу авторевью с prompt, замечания на PR            | шаг 2 (ci:-PR) + шаг 3 (synchronize) |
| 5 | Коммиты по Conventional Commits; release-PR после мержа | этот шаг, действия 1 и 3             |

  Плюс «Результат шага» спеки: цикл пройден, авторевью работает на
  каждый PR, release-please собирает release-PR.

## Файлы

- Create/Modify: ничего руками; release-PR — артефакт release-please.
- Не мержить release-PR: его судьба — за пределами шага (спека требует
  «найдите release-PR»).

## Проверка (real-run)

| # | Команда                                             | Ожидаемый результат                             |
| - | --------------------------------------------------- | ----------------------------------------------- |
| 1 | `gh pr view <N> --json commits`                     | headlines по Conventional Commits, есть `feat:` |
| 2 | `gh pr view <N> --json state -q .state`             | `MERGED`                                        |
| 3 | `git log --oneline origin/main -5`                  | feat-коммиты агента в `main` (merge --merge)    |
| 4 | `gh run list --workflow "Release Please" --limit 1` | `success`                                       |
| 5 | `gh pr list --state open`                           | release-PR `chore(main): release 0.2.0`         |
| 6 | `gh pr view <R> --json files`                       | CHANGELOG.md + манифест версии                  |
| 7 | `gh issue view 7 --json state -q .state`            | `CLOSED` (автозакрытие по `Closes #7`)          |
| 8 | `gh run list --branch main --limit 4`               | предсуществующие workflows — `success`          |

## Отрицательные проверки

- `opencode-review` не запускался на открытие release-PR
  (GITHUB_TOKEN-нетриггер): `gh run list --workflow opencode-review` —
  без `opened` по release-PR.
- Release-PR не смержен:

  ```bash
  gh pr view <R> --json state -q .state
  ```

  Ожидание: `OPEN`.

- В `main` нет прямых пушей мимо PR:

  ```bash
  git log --graph --oneline -5
  ```

  Ожидание: merge-коммит и его родители, без висячих прямых коммитов.

## Коммит

Коммитов шага нет: мерж существующего PR; release-PR — артефакт
release-please. Cycle-report — отдельный контекст-файл после приёмки
(по образцу task-8/9), следующий `docs:`-коммит, в объём шага не
входит.

## Результат шага (description.md)

Цикл «issue → pull request → ревью → правки → мерж» пройден с участием
агента; коммиты агента по Conventional Commits; после мержа
release-please собрал release-PR. Авторевью работает на каждый pull
request (шаги 2–3; опционально reopen-доказательство здесь).

## Источники

- real-run (fetch, 2026-10-06): release-please (googleapis/
  release-please-action) — simple release type: разбор Conventional
  Commits, release-PR вида «chore(main): release X.Y.Z» с changelog и
  бампом версии; feat → minor. Конфигурация репо уже настроена
  (task-1), в шаге не меняется.
- real-run (shell, 2026-10-06): `cat .release-please-manifest.json`
  (`0.1.0`); `cat release-please-config.json` (`simple`);
  `gh pr list --state all` (release-PR ранее не создавался — здесь
  создастся впервые); `gh pr merge --merge` — конвенция репо (merge
  коммиты #6, #8 в истории).
- real-run (чтение файлов, 2026-10-06): `.github/workflows/release-please.yml`
  (push → `main`, без `with:` — дефолтные конфиг-файлы корня);
  description.md (требование 5, задача 6, результат шага); AGENTS.md
  (Conventional Commits); task-9 cycle report (формат приёмочной
  таблицы и хронологии).
- code-reading: GITHUB_TOKEN-созданные PR не триггерят
  pull_request-события — документированное поведение GitHub Actions
  (исключения: workflow_dispatch, repository_dispatch); reopen как
  детерминированный триггер — тип `reopened` в списке событий
  воркфлоу; merge `--merge` сохраняет индивидуальные коммиты — при
  squash release-please разбирал бы заголовок сквоша, а не коммиты
  агента.
