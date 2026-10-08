# step-5-release-please

- Дата: 2026-09-29 07:02
- Ветка: main
- Шаг: план шага 5/6 — release-please

Исходный план: `20260929062503-implementation-plan.md`.

## Цель

Автоматические релизы: release-please читает историю Conventional Commits,
собирает changelog и держит release-PR с версией по семантическому
версионированию после мержа в основную ветку.

## Действия

1. `.github/workflows/release-please.yml`:

   - `on: push` в `main`
   - permissions: contents: write, pull-requests: write
   - google-github-actions/release-please-action@v4

2. `.release-please-manifest.json` в корне: корневой релиз, ключ `""`,
   стартовая версия `0.1.0`
3. `release-please-config.json`: минимум — `{"release-type": "simple"}`

## Файлы

- `.github/workflows/release-please.yml` — новый
- `.release-please-manifest.json` — новый
- `release-please-config.json` — новый

## Проверка

| # | Действие (real-run на GitHub)   | Ожидаемый результат              |
| - | ------------------------------- | -------------------------------- |
| 1 | смержить feat/fix-коммит в main | открывается release-PR           |
| 2 | смержить release-PR             | создаётся тег vX.Y.Z и changelog |

## Блокеры

- GitHub-remote не настроен — проверка только после подключения
  remote (open question №1 из исходного плана)
- Для открытия release-PR в истории уже должны быть Conventional
  Commits (появятся после шагов 1–3)

## Коммит (делает пользователь)

`chore: add release-please workflow`

## Источники

- code-reading: `step-1.md` — release-please отдельным workflow,
  release-PR после мержа в основную ветку
