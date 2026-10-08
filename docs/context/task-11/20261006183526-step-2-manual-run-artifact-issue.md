# step-2-manual-run-artifact-issue

- Дата: 2026-10-06 18:35 (+05)
- Ветка: без рабочей ветки и коммитов — всё через GitHub (запуск
  воркфлоу, вкладки Actions и Issues); при необходимости фикса — новый
  `ci:`-PR (паттерн task-9 «докоммит»)
- Шаг: план шага 2/3 — ручной запуск `opencode-scheduled`
  (`workflow_dispatch`), зелёный прогон: warm-up → Lighthouse →
  артефакт → агент → issue с находками

Исходный план: `20261006183526-implementation-plan.md`.

## Цель

Один успешный прогон `opencode-scheduled` по `workflow_dispatch`:
warm-up вернул `200`; шаг Lighthouse создал `lighthouse/report.json`;
артефакт `lighthouse-report` виден во вкладке Artifacts прогона;
агент создал ровно одну issue с датой в заголовке, оценками категорий
и рекомендациями. Это закрывает требования 4 и 5 спеки («хотя бы один
успешный прогон», «отчёт артефактом», «минимум один issue по
находкам»).

## Interfaces

- Consumes: воркфлоу `opencode-scheduled` в `main` + переменная
  `APP_URL` (шаг 1); публичный деплой `https://calendar-387.onrender.com`;
  секрет `ZHIPU_API_KEY`.
- Produces: номер успешного прогона, имя артефакта `lighthouse-report`,
  номер issue с находками, номер триаж-комментария на ней — материал
  приёмки шага 3 и cycle-report.

## Предусловия (real-run перед стартом)

- [ ] 1. Файл воркфлоу в `main` и воркфлоу active — иначе
  `workflow_dispatch` недоступен (docs: trigger only on default
  branch):

  ```bash
  git fetch
  git show origin/main:.github/workflows/opencode-scheduled.yml >/dev/null && echo MAIN-OK
  gh workflow list | grep scheduled
  ```

  Ожидание: `MAIN-OK`; `opencode-scheduled … active`.

- [ ] 2. Переменная задана, приложение отвечает:

  ```bash
  gh variable list
  curl -s -o /dev/null -w "%{http_code}\n" --max-time 90 \
    "https://calendar-387.onrender.com"
  ```

  Ожидание: `APP_URL` в списке; `200`.

## Действия

- [ ] 1. Запустить воркфлоу вручную:

  ```bash
  gh workflow run opencode-scheduled
  sleep 10
  gh run list --workflow opencode-scheduled --limit 1
  ```

  Ожидание: появилась запись прогона со статусом `in_progress`/
  `queued`, event `workflow_dispatch`. Альтернатива через UI:
  Actions → opencode-scheduled → Run workflow.

- [ ] 2. Дождаться завершения и разобрать лог по шагам:

  ```bash
  gh run watch $(gh run list --workflow opencode-scheduled --limit 1 --json databaseId -q '.[0].databaseId')
  gh run view $(gh run list --workflow opencode-scheduled --limit 1 --json databaseId -q '.[0].databaseId') --log | grep -E "attempt|HTTP|Warm" | head -20
  ```

  Ожидание: итог `success`. В логе warm-up-шага — серии
  `attempt N: HTTP 200` (первый запрос мог занять десятки секунд —
  cold start, warm-up отработал); в шаге Lighthouse — отсутствие
  ошибок Chrome (fallback при ошибке браузера: `export
  CHROME_PATH=/usr/bin/google-chrome-stable` докоммитом `ci:`).

- [ ] 3. Найти артефакт отчёта (требование 4 спеки):

  ```bash
  gh run view $(gh run list --workflow opencode-scheduled --limit 1 --json databaseId -q '.[0].databaseId')
  gh api repos/:owner/:repo/actions/runs/<RUN_ID>/artifacts -q '.artifacts[].name'
  ```

  Ожидание: во вкладке Artifacts прогона — `lighthouse-report`.
  Скачивание и глазная проверка структуры JSON:

  ```bash
  gh run download <RUN_ID> -n lighthouse-report -D /tmp/opencode/lh-check
  python3 -c "
  import json,glob
  p=glob.glob('/tmp/opencode/lh-check/**/report.json',recursive=True)[0]
  d=json.load(open(p))
  print(d['fetchTime'])
  for k,v in d['categories'].items(): print(k, v['score'])
  "
  ```

  Ожидание: JSON валиден, `fetchTime` свежий, четыре категории с
  score 0..1.

- [ ] 4. Найти issue от агента (требование 5 спеки):

  ```bash
  gh issue list --state open --limit 10
  ```

  Ожидание: новая issue с заголовком `Lighthouse: регулярная проверка
  <дата прогона YYYY-MM-DD>`; автор — app-идентичность агента; в теле
  — оценки категорий и рекомендации.

- [ ] 5. Прочитать issue глазами — качество находок:

  Проверить: (а) оценки категорий совпадают с JSON из артефакта (шаг 3);
  (б) топ-аудиты взяты из отчёта, не выдуманы; (в) рекомендации
  конкретны или честно помечены «источник неочевиден». Расхождение
  цифр с JSON — сигнал, что агент читал не тот файл или галлюцинирует:
  усилить prompt (декоммит `ci:` — правка пункта «тело issue»).

- [ ] 6. Зафиксировать побочный триаж (наблюдение, не дефект):

  ```bash
  gh issue view <ISSUE_NUMBER> --comments | head -40
  gh run list --workflow opencode-triage --limit 2
  ```

  Ожидание: на новой issue — триаж-комментарий от существующего
  конвейера (`opencode-triage.yml` слушает `issues: opened`);
  прогон триажа — `success`. Зацикливания нет: комментарий триажа без
  `/oc` не будит `opencode.yml` (task-9). Материал — в cycle-report.

- [ ] 7. (Отрицательная проверка) Повторный запуск того же дня —
  осознанный дубль:

  ```bash
  gh workflow run opencode-scheduled
  ```

  Запускать только при желании проверить идемпотентность заголовка:
  дата в заголовке одинакова → вторая issue с тем же заголовком
  появится (дедупликация — открытый вопрос плана). Для минимальной
  спеки достаточно одного прогона — шаг опционален, решение
  пользователя.

## Файлы

- Create: ничего в репозитории (прогоны создают артефакты и issue на
  стороне GitHub).
- Modify: ничего. Фикс-сценарий — только новый `ci:`-PR с точечной
  правкой `opencode-scheduled.yml` (не ломать шаг 1 ретроспективно).
- Не трогать: код, контракт, README, `hexlet-check.yml`.

## Проверка (real-run)

| # | Команда                                                          | Ожидаемый результат                                        |
| - | ---------------------------------------------------------------- | ---------------------------------------------------------- |
| 1 | `gh run list --workflow opencode-scheduled --limit 1`            | `completed`, `success`, event `workflow_dispatch`          |
| 2 | лог warm-up-шага                                                 | серии `attempt N: HTTP 200` до старта Lighthouse           |
| 3 | `gh api …/runs/<RUN_ID>/artifacts -q '.artifacts[].name'`        | `lighthouse-report`                                        |
| 4 | `gh run download` + python-разбор JSON                           | валидный JSON, `fetchTime` прогона, 4 категории            |
| 5 | `gh issue list --state open`                                     | issue `Lighthouse: регулярная проверка <дата>`             |
| 6 | `gh run list --workflow opencode-triage --limit 2`               | success на `issues` event новой issue                      |

## Отрицательные проверки

- Адрес приложения в файле воркфлоу не появился (правки не делались):
  локального диффа нет — `git status --porcelain` пуст.
- Значения секретов в логах прогона: `gh run view --log | grep -i
  "zhipu\|sk-"` — только имена/маски, не значения (GitHub маскирует
  секреты автоматически; зрительная проверка лога шага Run OpenCode).
- У агента не появилось веток/PR от этого прогона (prompt запрещает):

  ```bash
  gh pr list --state open
  git ls-remote --heads origin | grep -i opencode
  ```

  Ожидание: открытых PR нет; веток `opencode/*` от schedule-прогона нет.

## Коммит

Без коммитов. Фикс-сценарий (warm-up/Lighthouse/prompt не сработали):
новая ветка от свежего `main`, точечная правка
`opencode-scheduled.yml`, `ci:`-коммит, PR → `main`, мерж, повторный
запуск этого шага с предусловий.

## Результат шага (description.md)

Есть хотя бы один успешный прогон воркфлоу; отчёт проверки сохранён
артефактом прогона (`lighthouse-report`) и продублирован выводами в
issue; по находкам заведена минимум одна issue. Требования 4 и 5
спеки закрыты живыми доказательствами.

## Источники

- real-run (fetch, 2026-10-06): opencode.ai/docs/github — Supported
  Events (`workflow_dispatch`: «Trigger OpenCode on demand via Actions
  tab. Requires prompt input. Output goes to logs and PRs» — создание
  issue не дефолт, поэтому прямая инструкция в prompt; Review Focus 3
  плана), Schedule Example (write-права на issues); docs.github.com —
  `on.workflow_dispatch` (trigger only on default branch — предусловие
  1 этого шага).
- real-run (shell, 2026-10-06): `gh run list` / `gh run watch` /
  `gh run view --log` (формат вывода для проверок 1–2); `gh api
  repos/:owner/:repo/actions/runs/<id>/artifacts` и `gh run download
  -n` (механика проверки артефакта); `gh issue list --state open`
  (текущий список issues — база для поиска новой); `curl` публичного
  URL (HTTP 200; cold start реален — warm-up в логе обязан показать
  серию попыток).
- real-run (чтение файлов, 2026-10-06): `.github/workflows/
  opencode-scheduled.yml` (структура шагов — что искать в логе:
  имена шагов Warm up / Collect Lighthouse / Upload artifact / Run
  OpenCode); `.github/workflows/opencode-triage.yml` (событие
  `issues: opened` — предсказание побочного триажа); README
  (in-memory хранилище Render — данные между деплоями обнуляются,
  на проверку Lighthouse не влияет, но помнить при разборе находок).
- code-reading: python-разбор скачанного JSON (`categories[].score`
  0..1, `fetchTime`) — стабильная схема отчёта Lighthouse v13; сверка
  цифр issue с JSON — критерий честности находок агента; дубль issue
  при повторном dispatch того же дня — следствие подстановки даты в
  заголовок (одинаков в пределах суток), принято осознанно.
