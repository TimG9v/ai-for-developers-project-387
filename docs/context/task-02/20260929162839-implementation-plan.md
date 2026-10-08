# implementation-plan

- Дата: 2026-09-29 16:28
- Ветка: main
- Шаг: план реализации

## Задача

Шаг 2 «Скиллы и агентная среда» (`context/tmp/task-2/descripton.md`): подключить
к репозиторию набор инженерных скиллов Мэтта Покока (`mattpocock/skills`) и
настроить его под проект скиллом `setup-matt-pocock-skills`. Ответы на вопросы
настройки фиксированы шагом: трекер — GitHub Issues репозитория проекта, метки
разбора — по умолчанию, документы предметной области — один контекст
(`CONTEXT.md` + `docs/adr/` в корне). Общий контекст проекта —
`context/tmp/description.md`.

## Механика (проверено real-run 2026-09-29 и по документации)

- CLI `skills` (vercel-labs/skills): `npx skills@latest add mattpocock/skills`
  из корня проекта ставит скиллы в `.agents/skills/<name>/SKILL.md` (38 скиллов,
  плоская раскладка) и пишет `skills-lock.json` в корень. Проверено пробной
  установкой в песочнице `/tmp/opencode/skills-probe` (real-run). AGENTS.md и
  CLAUDE.md CLI не создаёт и не правит.
- OpenCode загружает проектные скиллы из `.agents/skills/<name>/SKILL.md` при
  старте новой сессии (opencode.ai/docs/skills) — после установки нужна новая
  сессия.
- `setup-matt-pocock-skills` входит в набор: исследует репозиторий, задаёт
  вопросы секциями (трекер → метки → доменные документы), показывает черновик,
  затем пишет `docs/agents/*.md` и блок `## Agent skills` в AGENTS.md. Прочитан
  его SKILL.md и сид-шаблоны (real-run, `~/.agents/skills/`).
- Скилл `triage` входит в набор → секция меток запускается,
  `docs/agents/triage-labels.md` пишется.

## Текущее состояние репозитория (real-run 2026-09-29)

| Что                            | Состояние                                         |
| ------------------------------ | ------------------------------------------------- |
| Ветка / рабочее дерево         | `main`, чистое                                    |
| remote                         | `github.com:TimG9v/ai-for-developers-project-386` |
| `docs/agents/`                 | отсутствует                                       |
| Раздел про скиллы в AGENTS.md  | нет (файл 27 строк)                               |
| `.agents/`, `skills-lock.json` | нет в репозитории                                 |
| mattpocock/skills глобально    | `~/.agents/skills/` — 86 скиллов, вне репо        |
| `gh` CLI                       | не установлен                                     |

## Ответы на вопросы setup (из descripton.md)

1. Трекер задач: GitHub Issues в репозитории проекта.
2. Метки для разбора задач: оставить значения по умолчанию.
3. Документы предметной области: один контекст — `CONTEXT.md` и `docs/adr/`
   в корне репозитория.

## Структура (что появится после шага)

```text
calendar/
├── .agents/skills/<name>/SKILL.md   # 38 скиллов набора mattpocock/skills
├── skills-lock.json                 # версии установленных скиллов
├── AGENTS.md                        # + раздел «Agent skills»
└── docs/agents/
    ├── issue-tracker.md             # GitHub Issues через gh CLI
    ├── triage-labels.md             # 5 канонических меток 1:1
    └── domain.md                    # single-context: CONTEXT.md + docs/adr/
```

## Последовательность работ (каждый пункт — отдельный Conventional Commit)

1. `chore:` — установка набора скиллов в проект: курс-команда
   `npx skills@latest add mattpocock/skills` (интерактивно: агент opencode,
   все скиллы) либо детерминированно
   `npx -y skills@latest add mattpocock/skills -a opencode --skill '*' -y`;
   ревью содержимого (скиллы исполняются с полными правами агента); проверка
   видимости в новой сессии OpenCode.
2. `docs:` — `/setup-matt-pocock-skills` с ответами выше →
   `docs/agents/{issue-tracker,domain,triage-labels}.md` + блок
   `## Agent skills` в AGENTS.md.
3. Приёмка шага — без коммита: чек-лист трёх требований descripton.md.

## Открытые вопросы (не блокируют план)

1. Коммиты делает пользователь — у агента git read-only.
2. Коллизия имён: в `~/.agents/skills` глобально лежит тот же набор — после
   установки проверить в новой сессии, чей источник выигрывает; при конфликте
   пользователь решает, убирать ли глобальный дубль (`npx skills remove -g`).
3. `gh` CLI не установлен: нужен не для настройки, а для будущей работы с
   тикетами (`gh issue create` и т. п.) — установить до шагов, использующих
   трекер.
4. Безопасность: в трёх скиллах набора есть shell-скрипты
   (git-guardrails-claude-code, wizard, diagnosing-bugs) — просмотреть перед
   коммитом; CLI прямо предупреждает: «Review skills before use; they run with
   full agent permissions».

## Критерии приёмки (из descripton.md)

- Набор скиллов установлен и доступен в агенте.
- В репозитории есть `docs/agents/` с записанной конфигурацией трекера, меток
  и документов предметной области.
- В AGENTS.md появился раздел про скиллы.

## Источники выводов

- real-run: состояние репозитория (git / ls / grep, 2026-09-29); пробная
  установка CLI в песочнице `/tmp/opencode/skills-probe` — раскладка, счётчик
  скиллов, поведение `npx skills list`; чтение SKILL.md и сид-шаблонов
  `setup-matt-pocock-skills` из `~/.agents/skills/`.
- code-reading: `context/tmp/task-2/descripton.md` — требования и ответы;
  `context/tmp/description.md` — общий контекст проекта.
- документация (web): README vercel-labs/skills — пути установки и флаги;
  opencode.ai/docs/skills — discovery проектных скиллов OpenCode.
