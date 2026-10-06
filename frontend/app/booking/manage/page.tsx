import { BookingManage } from "@/components/booking-manage";

// Запись зависит от id в query и меняется в рантайме (in-memory хранилище) —
// статический пререндер запёк бы пустой ответ на этапе сборки.
export const dynamic = "force-dynamic";

/**
 * Управление записью: гость отменяет и переносит запись по ссылке из
 * подтверждения (`/booking/manage?id=<bookingId>`), без авторизации
 * (АDR 0002 — API открыт, id записи фактически unguessable-токен).
 */
export default async function BookingManagePage({
  searchParams,
}: {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}) {
  const { id } = await searchParams;
  const bookingId = typeof id === "string" ? id : "";

  return (
    <main className="mx-auto flex w-full max-w-2xl flex-1 flex-col items-center gap-6 px-6 py-16">
      <h1 className="text-3xl font-semibold tracking-tight">
        Управление записью
      </h1>
      <BookingManage bookingId={bookingId} />
    </main>
  );
}
