"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

import { bookingsCancel } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

// Сколько кнопка ждёт повторного клика, прежде чем вернуться в исходное
// состояние: случайный клик не должен успеть «дожить» до подтверждения.
const CONFIRM_TIMEOUT_MS = 4000;

/**
 * Отмена записи владельцем из админки (история 7): тот же контрактный
 * DELETE /bookings/{id}, что и у гостя по ссылке управления. Отмена
 * необратима (in-memory хранилище, слот тут же уходит другим гостям),
 * поэтому клик двухшаговый: «Отменить» → «Точно отменить?»; без повторного
 * клика в течение CONFIRM_TIMEOUT_MS кнопка сама возвращается в исходное
 * состояние. После отмены список предстоящих встреч перезагружается
 * на сервере (force-dynamic).
 */
export function AdminCancelBooking({ bookingId }: { bookingId: string }) {
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!confirming) {
      return;
    }
    const timer = setTimeout(() => setConfirming(false), CONFIRM_TIMEOUT_MS);
    return () => clearTimeout(timer);
  }, [confirming]);

  function askConfirmation() {
    setError(null);
    setConfirming(true);
  }

  async function handleCancel() {
    setPending(true);
    setError(null);
    const { error: cancelError } = await bookingsCancel({
      path: { id: bookingId },
    });
    setPending(false);
    setConfirming(false);
    if (cancelError !== undefined) {
      setError("Не удалось отменить запись");
      return;
    }
    router.refresh();
  }

  return (
    <div className="flex flex-col items-start gap-1">
      <button
        type="button"
        onClick={() => void (confirming ? handleCancel() : askConfirmation())}
        disabled={pending}
        aria-label={
          confirming
            ? `Подтвердите отмену записи ${bookingId}`
            : `Отменить запись ${bookingId}`
        }
        className={
          confirming
            ? "rounded-lg border bg-background px-4 py-2 text-destructive"
            : "rounded-lg border bg-background px-4 py-2"
        }
      >
        {pending ? "Отменяем…" : confirming ? "Точно отменить?" : "Отменить"}
      </button>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
