# step-2-readme-process-docs

- Дата: 2026-10-08 21:31 (+05)
- Ветка: `task-12` — та же рабочая ветка, после мержа шага 1
  fast-forward на `origin/main`; мерж через PR, `--merge`
- Шаг: план шага 2/3 — README: раздел «Агентные воркфлоу» (таблица
  девяти воркфлоу, принятые решения, самооценка работы с агентом);
  PR `docs:` → `main`, мерж

Исходный план: `20261008213145-implementation-plan.md`.

## Цель

В `README.md` замержен раздел про агентный процесс: таблица всех девяти
воркфлоу проекта (событие, модель, назначение, где смотреть прогоны),
принятые решения (команда `/oc`, круг вызывающих, бот-фильтры,
`share: false`, права и расходы) и короткая самооценка работы с
агентом (что закрыто с первого прохода, где потребовались итерации) —
требования 2, 4, 5, 6, 7 спеки в части записей.

## Interfaces

- Consumes: состояние воркфлоу после шага 1 (бот-фильтры, `mentions:
  /oc`, `share: false`, `issues: read`); материалы cycle-reports
  task-8–11 (прогоны, PR, инциденты — для самооценки); существующий
  абзац CI в README (строки 137–140, `main`).
- Produces: раздел «Агентные воркфлоу» в `main` — вход шага 3
  (приёмка требований 2/4/5/6/7) и точка входа для нового человека
  в проекте (цель спеки).

## Предусловия (real-run перед стартом)

- [ ] 1. Шаг 1 замержен: три правки в `main`, воркфлоу активны:

  ```bash
  git checkout task-12 && git fetch
  git merge --ff-only origin/main
  git show origin/main:.github/workflows/opencode-triage.yml | grep -c "user.type != 'Bot'"
  gh workflow list | grep opencode
  ```

  Ожидание: merge без конфликтов; в main ≥ 1 совпадение; четыре
  opencode-воркфлоу `active`.

- [ ] 2. Актуальный README без раздела агента:

  ```bash
  grep -n "Агентные воркфлоу" README.md
  ```

  Ожидание: пусто (exit 1).

## Действия

- [ ] 1. Вставить раздел «Агентные воркфлоу» в README.md сразу после
  абзаца CI («CI — GitHub Actions на каждый push и PR: … release-please
  (release-PR после мержа в `main`).»), перед «## Известные ограничения»:

  ````markdown
  ## Агентные воркфлоу

  Агент OpenCode подключён через GitHub App (OIDC) и работает в четырёх
  воркфлоу. Модель везде `zai-coding-plan/glm-5.3-flash`: разбор,
  триаж и ревью не требуют тяжёлых рассуждений, плановая квота
  провайдера делает прогоны практически бесплатными; `variant`
  (уровень рассуждений) не задан. Прогоны и ответы агента смотрятся
  во вкладке Actions и в комментариях к issue/PR.

| Воркфлоу           | Запуск                           | Модель                        | Назначение                             | Где смотреть прогон                          |
| ------------------ | -------------------------------- | ----------------------------- | -------------------------------------- | -------------------------------------------- |
| Backend CI         | push и PR                        | —                             | fmt, clippy, тесты backend             | Actions → Backend CI                         |
| Frontend CI        | push и PR                        | —                             | eslint, vitest, build                  | Actions → Frontend CI                        |
| Security           | push и PR                        | —                             | cargo/npm audit по lock-файлам         | Actions → Security                           |
| hexlet-check       | push                             | —                             | Docker-сборка, автотесты Хекслета      | Actions → hexlet-check                       |
| Release Please     | push в `main`                    | —                             | release-PR: changelog и semver         | Actions → Release Please                     |
| opencode           | комментарий `/oc` в issue или PR | zai-coding-plan/glm-5.3-flash | разбор задачи, фиксы и фичи по команде | Actions, ответ — комментарий в треде         |
| opencode-triage    | открыта issue (не ботом)         | zai-coding-plan/glm-5.3-flash | автотриаж: причина, путь исправления   | Actions, ответ — комментарий к issue         |
| opencode-review    | PR открыт/обновлён (не бот)      | zai-coding-plan/glm-5.3-flash | авторевью PR по правилам AGENTS.md     | Actions, ответ — комментарий к PR            |
| opencode-scheduled | cron 06:00 МСК + вручную         | zai-coding-plan/glm-5.3-flash | Lighthouse деплоя, issue по находкам   | Actions, артефакт `lighthouse-report`, issue |

  ### Принятые решения

  - **Команда вызова одна — `/oc`** (параметр `mentions`): дефолтная
    пара `/opencode,/oc` сужена — подстрока `/opencode` встречается
    в ссылках `opencode.ai` и запускала воркфлоу на комментарии бота.
  - **Звать агента могут только участники с правом записи** — условие
    `author_association` (OWNER/COLLABORATOR/MEMBER) в воркфлоу:
    посторонний `/oc`-комментарий не запускает даже прогона.
  - **События от ботов отсечены** во всех воркфлоу с автором события
    (`user.type != 'Bot'`): у триажа — автор issue, у ревью — автор PR,
    у `/oc` — автор комментария. У schedule/dispatch автора нет —
    их защита: `timeout-minutes` и `concurrency`.
  - **Сессии не публикуются**: `share: false` во всех четырёх воркфлоу.
    Публичный репозиторий: дефолт экшена публикует сессию с полным
    контекстом по ссылке — решение закрыто явно.
  - **Права минимальны и раздельны**: интерактивный воркфлоу — только
    `id-token: write` (в репозиторий агент пишет через App-токен);
    триаж — плюс `issues: write` (комментарии); ревью — `contents: read`,
    `pull-requests: write`, `issues: read` (пишет только в PR);
    расписание — write-набор целиком: issue по находкам плюс
    output-канал экшена, который сам оформляет файлы прогона PR'ом.
  - **Расход под контролем**: timeouts 20–45 минут и `concurrency`
    против «тихих» зависаний; дешёвая модель; прогоны на комментариях
    без `/oc` — skipped и бесплатны.

  ### Самооценка работы с агентом

  Закрылось с первого прохода: разбор issue по `/oc` с причиной по
  коду (task-8), автотриаж новой issue по структуре prompt (task-9),
  ночной Lighthouse-конвейер — артефакт и issue по находкам совпали
  с сырым отчётом без галлюцинаций (task-11).

  Потребовало итераций: связка модель/ключ — Zen не признал ключ
  Coding Plan (4 падения AuthError, переход на `zai-coding-plan`,
  PR #6); самопробуждение воркфлоу на подстроку `/opencode` в ответах
  бота (лечится бот-фильтром и узким `mentions`); «тихие» зависания
  прогонов (55+ минут — добавлены timeout и concurrency); формат
  коммитов в агентных PR (повторная итерация с `--force-with-lease`,
  task-10); конфиг release-please (три инфраструктурных починки,
  task-10); сайд-эффект scheduled-прогона — PR с `report.json`
  (закрыт, канал задокументирован, task-11).
  ````

- [ ] 2. Сверка таблицы с фактами (`main`): имена воркфлоу — из
  `gh workflow list`; события и права — из файлов; модель — из
  `model:` в четырёх файлах; cron — из `opencode-scheduled.yml`:

  ```bash
  gh workflow list
  grep -H "model:" .github/workflows/opencode*.yml
  grep -H "cron:" .github/workflows/opencode-scheduled.yml
  ```

  Ожидание: 9 воркфлоу; четыре строки `zai-coding-plan/glm-5.3-flash`;
  cron `0 6 * * *`.

- [ ] 3. Локальная проверка README:

  ```bash
  grep -n "Агентные воркфлоу\|Принятые решения\|Самооценка" README.md
  awk 'length($0) > 180 {print FNR}' README.md
  ```

  Ожидание: три заголовка на месте; строк длиннее 180 символов нет.

- [ ] 4. Точечный коммит (только README):

  ```bash
  git add README.md
  git status --porcelain
  git commit -m "docs: описать агентные воркфлоу, решения и самооценку в README"
  ```

  Ожидание: в индексе один файл; Conventional Commit `docs:`.

- [ ] 5. PR → `main`, проверки, мерж:

  ```bash
  git push -u origin task-12
  gh pr create --title "docs: описать агентные воркфлоу в README" --fill
  gh pr checks --watch
  gh pr merge --merge
  ```

  Ожидание: проверки зелёные (opencode-review оставит замечания к
  собственному тексту — принять или отработать докоммитом; мерж
  не блокировать без блокирующих замечаний).

- [ ] 6. Проверка в `main`:

  ```bash
  git fetch
  git show origin/main:README.md | grep -c "opencode-scheduled"
  git diff origin/main...HEAD --name-only
  ```

  Ожидание: раздел в `main`; дифф PR — только README.

## Файлы

- Modify: `README.md` — новый раздел после абзаца CI.
- Не трогать: воркфлоу (шаг 1 завершён), `hexlet-check.yml`, код,
  контракты.

## Проверка (real-run)

| # | Команда                                                                     | Ожидаемый результат                                             |
| - | --------------------------------------------------------------------------- | --------------------------------------------------------------- |
| 1 | `git show origin/main:README.md \| grep -n "Агентные воркфлоу"`             | заголовок раздела в `main`                                      |
| 2 | таблица в README (взглядом на GitHub)                                       | 9 строк, колонки: воркфлоу, запуск, модель, назначение, прогоны |
| 3 | `git show origin/main:README.md \| grep -c "zai-coding-plan/glm-5.3-flash"` | ≥ 4 (в таблице + абзац)                                         |
| 4 | раздел «Принятые решения»                                                   | 6 пунктов: /oc, круг, боты, share, права, расход                |
| 5 | раздел «Самооценка работы с агентом»                                        | два блока: с первого прохода / итерации, с номерами PR          |
| 6 | `gh pr view --json state -q .state` (URL PR)                                | `MERGED`                                                        |

## Отрицательные проверки

- Раздел не трогает соседние:

  ```bash
  git show origin/main:README.md | grep -n "^## "
  ```

  Ожидание: прежние заголовки на месте, «Агентные воркфлоу» — один
  новый между «Разработка» и «Известные ограничения».

- В README нет значений секретов и ссылок на приватное:

  ```bash
  git show origin/main:README.md | grep -E "ZHIPU|RENDER_API|ghp_|sk-"
  ```

  Ожидание: пусто (exit 1).

- Маркировка решений согласована с кодом шага 1:

  ```bash
  git show origin/main:README.md | grep -c "share: false"
  git show origin/main:README.md | grep -c "/oc"
  ```

  Ожидание: `share: false` упомянут; `/oc` — единственная команда
  вызова, `/opencode` как команда не упоминается (только как история
  инцидента).

## Коммит

Один коммит: `docs: описать агентные воркфлоу, решения и самооценку
в README` (файл `README.md`). В `main` — через PR, мерж `--merge`.

## Результат шага (description.md)

В README описаны воркфлоу проекта: чем запускаются, какая модель
используется и где смотреть прогоны (таблица); решение по share
принято и записано; набор команд и круг вызывающих задокументированы;
самооценка работы с агентом записана коротко и с пруфами. Живая
проверка всего — шаг 3.

## Источники

- real-run (shell/чтение, 2026-10-08): `gh workflow list` (имена и
  статусы девяти воркфлоу); четыре файла `opencode*.yml` (события,
  модель, cron, права); README.md (абзац CI, строки 137–140; структура
  разделов); переменная `APP_URL` и публичный деплой — README.
- real-run (архив task-8–11, чтение отчётов): cycle-report task-8
  (прогоны 37395173305 success; AuthError ×4; фикс PR #6; самопробуждение);
  cycle-report task-9 (автотриаж #9 success 3m21s; skipped 1s без /oc);
  step-4-report task-10 (reword-итерация с --force-with-lease; три
  починки release-please; release 0.2.0); step-2-report task-11
  (зависание 37750627398; артефакт 72 144 байта; issue #26 — совпадение
  оценок с JSON; сайд-канал PR #27; триаж failure 37757599684).
- code-reading: формулировки решений — перенос в человеческий текст
  фактов шага 1 и docs Configuration (mentions/share); самооценка —
  агрегация отчётов, номера прогонов/PR сохранены как пруфы.
