# step-1-skills-install

- Дата: 2026-09-29 16:28
- Ветка: main
- Шаг: план шага 1/3 — установка набора скиллов в репозиторий

Исходный план: `20260929162839-implementation-plan.md`.

## Цель

Набор `mattpocock/skills` установлен в проект (`.agents/skills/` +
`skills-lock.json`), просмотрен перед коммитом и виден агенту в новой сессии
OpenCode.

## Действия

1. Из корня репозитория запустить установку:

   - курс-команда из шага (интерактивная; выбрать агента opencode и все
     скиллы набора):

     ```bash
     npx skills@latest add mattpocock/skills
     ```

   - детерминированный эквивалент для агента (те же выборы флагами;
     проверен в песочнице):

     ```bash
     npx -y skills@latest add mattpocock/skills -a opencode --skill '*' -y
     ```

   Повторный запуск безопасен: lock-файл хранит `computedHash` каждого скилла,
   повторная установка обновляет, а не дублирует.

2. Осмотреть результат:

   ```bash
   ls .agents/skills | wc -l                   # 38
   find .agents/skills -name SKILL.md | wc -l  # 38
   head -40 skills-lock.json
   ```

3. Ревью безопасности перед коммитом (скиллы исполняются с полными правами
   агента):

   - просмотреть список имён: `ls .agents/skills`;
   - выборочно прочитать пару скиллов:
     `head -30 .agents/skills/triage/SKILL.md`,
     `head -30 .agents/skills/implement/SKILL.md`;
   - shell-скрипты в составе набора (ожидаемо ровно три, просмотреть каждый):

     ```bash
     find .agents/skills -type f -name '*.sh'
     # .agents/skills/git-guardrails-claude-code/scripts/block-dangerous-git.sh
     # .agents/skills/wizard/template.sh
     # .agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh
     ```

4. Убедиться, что новые пути не попадают под gitignore:

   ```bash
   grep -nE '\.agents|skills-lock' .gitignore   # пустой вывод — ок
   ```

5. Проверить видимость агентом: открыть новую сессию OpenCode в корне репо —
   в списке доступных скиллов должны появиться скиллы набора (например
   `setup-matt-pocock-skills`, `triage`, `implement`). Текущая сессия не
   подойдёт: OpenCode читает `.agents/skills/` только на старте.
6. Посмотреть итог: `git status --short` — добавлены `.agents/` и
   `skills-lock.json`, других изменений нет.

## Файлы

- `.agents/skills/*` — 38 скиллов набора (каждый: каталог с SKILL.md)
- `skills-lock.json` — версии и источники установленных скиллов

## Проверка

| # | Команда (real-run)                          | Ожидаемый результат               |
| - | ------------------------------------------- | --------------------------------- |
| 1 | npx skills list                             | секция Project Skills, 38 записей |
| 2 | find .agents/skills -name SKILL.md \| wc -l | 38                                |
| 3 | новая сессия OpenCode в корне репо          | скиллы набора в списке доступных  |
| 4 | git status --short                          | .agents/ и skills-lock.json       |

## Коммит (делает пользователь)

```bash
git add .agents skills-lock.json
git commit -m "chore: add mattpocock engineering skills to the repo"
```

## Предусловия следующих шагов

Шаг 2 (настройка) выполняется в новой сессии OpenCode — к этому моменту
скиллы уже загружены из `.agents/skills/`.

## Источники

- real-run: пробная установка в `/tmp/opencode/skills-probe` (2026-09-29) —
  38 скиллов, плоская раскладка `.agents/skills/<name>/SKILL.md`,
  `skills-lock.json` в корне, AGENTS.md CLI не создаёт.
- code-reading: `context/tmp/task-2/descripton.md` — команда установки из
  текста шага.
- документация (web): README vercel-labs/skills — project scope
  `./.agents/skills/` для opencode, флаги `-a/-s/-y`, идемпотентность через
  lock-файл.
