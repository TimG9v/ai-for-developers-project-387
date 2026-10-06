"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { bookingsCancel } from "@/src/client";

// Same-origin конфиг SDK для браузера (тикет #20) — см. admin-event-types.
import "@/src/api-config";

/**
 * Отмена записи владельцем из админки (история 7): тот же контрактный
 * DELETE /bookings/{id}, что и у гостя по ссылке управления. После отмены
 * список предстоящих встреч перезагружается на сервере (force-dynamic).
 */
export function AdminCancelBooking({ bookingId }: { bookingId: string }) {
  const router = useRouter();
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleCancel() {
    setPending(true);
    setError(null);
    const { error: cancelError } = await bookingsCancel({
      path: { id: bookingId },
    });
    setPending(false);
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
        onClick={() => void handleCancel()}
        disabled={pending}
        aria-label={`Отменить запись ${bookingId}`}
        className="rounded-lg border bg-background px-4 py-2"
      >
        {pending ? "Отменяем…" : "Отменить"}
      </button>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
