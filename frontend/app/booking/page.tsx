import { eventTypesList } from "@/src/client";

import { BookingEventTypes } from "@/components/booking-event-types";

// Список типов встреч меняется в рантайме (in-memory хранилище) —
// статический пререндер запёк бы пустой список на этапе сборки.
export const dynamic = "force-dynamic";

export default async function BookingPage() {
  const { data } = await eventTypesList();
  const eventTypes = data ?? [];

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col items-center justify-center gap-4 px-6 py-32">
      <h1 className="text-3xl font-semibold tracking-tight">Страница записи</h1>
      <p className="text-muted-foreground">Выберите тип встречи</p>
      <BookingEventTypes initialEventTypes={eventTypes} />
    </main>
  );
}
