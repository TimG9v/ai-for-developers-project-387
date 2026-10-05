# step-2-implement-homepage

- Дата: 2026-09-29 18:04
- Ветка: task-3 (рабочая)
- Шаг: план шага 2/3 — реализация главной страницы с тестами через /implement

Исходный план: `20260929180440-implementation-plan.md`.

## Цель

Главная страница рассказывает про сервис и ведёт ссылкой на страницу записи
(`/booking`); страница записи — минимальная заглушка. Обе покрыты тестами, локальные
прогоны зелёные, `/code-review` пройдено.

## Вход из шага 1 (подставить фактические решения интервью)

- Словарь: `CONTEXT.md` (термины «главная страница», «страница записи»).
- Решения: `docs/adr/0001-*.md` (кандидат — язык интерфейса: русский).
- Ниже — вариант по рекомендациям плана. Если интервью решило иначе — заменить
  тексты, маршрут и язык на решения интервью; структура шага не меняется.

## Действия

1. Прочитать правило Next.js 16 (frontend/AGENTS.md) и гайд по ссылкам до правки
   кода: `frontend/node_modules/next/dist/docs/01-app/01-getting-started/04-linking-and-navigating.md`.
2. Запустить `/implement` (в той же сессии, что и интервью, — результат разговора
   ещё в контексте, артефакты — в репо) с задачей: «главная страница и заглушка
   страницы записи по решениям шага 1, тесты вместе с кодом».
3. TDD — сначала failing-тесты. Обновить `frontend/tests/smoke.test.tsx`: утверждение
   про кнопку «Create event» уходит вместе с заглушкой скаффолда, имя файла
   сохраняется (меньше шума в диффе):

   ```tsx
   import { render, screen } from "@testing-library/react";
   import { describe, expect, it } from "vitest";

   import Page from "@/app/page";

   describe("home page", () => {
     it("shows the service title and pitch", () => {
       render(<Page />);

       expect(
         screen.getByRole("heading", { level: 1, name: "Запись на звонок" }),
       ).toBeTruthy();
       expect(
         screen.getByText("Выберите свободное время и запишитесь на встречу"),
       ).toBeTruthy();
     });

     it("links to the booking page", () => {
       render(<Page />);

       const link = screen.getByRole("link", { name: "Записаться на звонок" });
       expect(link.getAttribute("href")).toBe("/booking");
     });
   });
   ```

   Проверка href — через `getAttribute`: матчер `toHaveAttribute` живёт в
   `@testing-library/jest-dom`, которого в зависимостях нет (real-run: package.json);
   новая зависимость не вводится.

4. Новый `frontend/tests/booking.test.tsx`:

   ```tsx
   import { render, screen } from "@testing-library/react";
   import { describe, expect, it } from "vitest";

   import Page from "@/app/booking/page";

   describe("booking page", () => {
     it("renders the page heading", () => {
       render(<Page />);

       expect(
         screen.getByRole("heading", { level: 1, name: "Страница записи" }),
       ).toBeTruthy();
     });
   });
   ```

5. Прогнать тесты, убедиться в падении: `cd frontend && npm test` — FAIL (страницы
   записи и ссылки ещё нет).
6. Реализация. `frontend/app/page.tsx`:

   ```tsx
   import Link from "next/link";

   import { Button } from "@/components/ui/button";

   export default function Home() {
     return (
       <main className="flex flex-1 flex-col items-center justify-center gap-6 py-32">
         <h1 className="text-3xl font-semibold tracking-tight">Запись на звонок</h1>
         <p className="max-w-md text-center text-muted-foreground">
           Выберите свободное время и запишитесь на встречу
         </p>
         <Button asChild>
           <Link href="/booking">Записаться на звонок</Link>
         </Button>
       </main>
     );
   }
   ```

   `Button asChild` рендерит `Slot.Root` (real-run: components/ui/button.tsx), ребёнок
   `Link` становится `<a>` со стилями кнопки — роль элемента `link`, что и проверяет
   тест.

7. Новый `frontend/app/booking/page.tsx` (заглушка — наполняется в следующих шагах
   курса):

   ```tsx
   export default function BookingPage() {
     return (
       <main className="flex flex-1 flex-col items-center justify-center gap-6 py-32">
         <h1 className="text-3xl font-semibold tracking-tight">Страница записи</h1>
         <p className="text-muted-foreground">
           Здесь появится выбор свободного времени
         </p>
       </main>
     );
   }
   ```

8. Метаданные приложения в `frontend/app/layout.tsx` — заменить scaffold-значение
   «Create Next App» (тот же коммит):

   ```tsx
   export const metadata: Metadata = {
     title: "Запись на звонок",
     description: "Выберите свободное время и запишитесь на встречу",
   };
   ```

9. Полный локальный прогон: `make test-frontend` (exit 0), `make lint-frontend`
   (exit 0), `cd frontend && npm run build` (exit 0 — в CI есть шаг Build, ловим
   проблемы локально до пуша).
10. `/code-review` по диффу шага; замечания исправить до коммита. Коммит делает
    пользователь.

## Файлы

- `frontend/app/page.tsx` — правка: контент главной + CTA-ссылка на `/booking`
- `frontend/app/booking/page.tsx` — новый: заглушка страницы записи
- `frontend/app/layout.tsx` — правка: metadata под проект
- `frontend/tests/smoke.test.tsx` — правка: утверждения главной (заголовок, текст,
  ссылка)
- `frontend/tests/booking.test.tsx` — новый: заглушка рендерится

## Проверка

| # | Команда (real-run)           | Ожидаемый результат                   |
| - | ---------------------------- | ------------------------------------- |
| 1 | cd frontend && npm test      | 3 passed (2 home + 1 booking), exit 0 |
| 2 | make lint-frontend           | exit 0                                |
| 3 | cd frontend && npm run build | exit 0                                |
| 4 | git status --short           | ровно 5 файлов шага                   |

## Коммит (делает пользователь)

```bash
git add frontend/app/page.tsx frontend/app/booking/page.tsx frontend/app/layout.tsx \
  frontend/tests/smoke.test.tsx frontend/tests/booking.test.tsx
git commit -m "feat(frontend): add homepage linking to booking page"
```

## Предусловия следующих шагов

Шаг 3 (приёмка) выполняется после пуша ветки — проверяет CI и итоговое состояние
репозитория целиком.

## Источники

- real-run (чтение): `frontend/app/page.tsx`, `layout.tsx`, `components/ui/button.tsx`
  (asChild → Slot.Root), `tests/smoke.test.tsx`, `vitest.config.ts` (jsdom, alias `@`),
  `package.json` (jest-dom отсутствует), `frontend-ci.yml` (шаг Build), путь гайда
  `04-linking-and-navigating.md` (ls node_modules/next/dist/docs).
- code-reading: требование «ведёт на страницу записи» — descripton.md; содержимое
  гайда Link читается на исполнении шага.
