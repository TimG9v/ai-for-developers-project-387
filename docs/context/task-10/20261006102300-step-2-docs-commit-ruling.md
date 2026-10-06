# step-2-docs-commit-ruling

- Дата: 2026-10-06 10:2x (+05)
- Шаг: ruling к шагу 2/4 — куда идёт `docs:`-коммит контекст-файлов

## Ruling

Действие 2 шага 2 (`git add docs/context/tmp/task-10 && git commit -m
"docs: add task-10 context docs"`) дефектно: `.gitignore` строка 9
(`tmp*`) игнорирует `docs/context/tmp/` целиком (проверено
`git check-ignore -v`, exit 0) — `git add` без `-f` отклонит путь, а
`-f` против конвенции (tmp — git-игнорируемый рабочий каталог).

По образцу task-9 (`e263a7f` «docs: add task-9 context docs» на ветке
`task-9`, mid-cycle): контекст-документы коммитятся в финальном месте
`docs/context/task-10/` — копия текущего набора из tmp, tmp остаётся
живым рабочим набором; поздние артефакты (cycle-report) переезжают
отдельным `docs:`-коммитом после приёмки (94441cd в task-9).

Правка действия 2 шага 2: создать ветку `task-10`, скопировать 6
файлов в `docs/context/task-10/`, `git add docs/context/task-10`,
`git commit -m "docs: add task-10 context docs"` (без push — push и PR
в действии 7 шага 2). Коммит выполняет пользователь
(no-git-commit-push); агенту выдан командный блок 2026-10-06.

Стоимость при ошибке: дублирование tmp/финальное место — принято
конвенцией task-9; расхождение копий лечится финальным переездом
после приёмки.

## Источники

- real-run (shell, 2026-10-06): `git check-ignore -v
  docs/context/tmp/task-10/...` (`.gitignore:9: tmp*`);
  `git ls-files docs/context/tmp/task-9/` (пусто — tmp не коммитился);
  `git ls-files docs/context/task-9/` (6+ файлов — финальное место);
  `git status --porcelain` (чисто — tmp невидим git'у).
- real-run (чтение): task-9 chronology в cycle report (e263a7f, 06:20,
  mid-cycle); `.gitignore`.
- code-reading: вывод «tmp — сознательно игнорируемый скретч» — из
  состава `.gitignore` и факта отсутствия tmp-путей в git-истории.
