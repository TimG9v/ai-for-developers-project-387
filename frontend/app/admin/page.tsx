import {
  upcomingMeetingsList,
  eventTypesList,
  type UpcomingMeeting,
} from "@/src/client";

import { AdminEventTypes } from "@/components/admin-event-types";
import { AdminSlots } from "@/components/admin-slots";
import { formatSlotInterval } from "@/lib/slot-time";

// Данные меняются в рантайме (in-memory хранилище).
export const dynamic = "force-dynamic";

type Meeting = {
  key: string;
  guestName: string;
  guestEmail: string;
  interval: string;
  eventTitle: string;
};

export default async function AdminPage() {
  const [eventTypesResult, meetingsResult] = await Promise.all([
    eventTypesList(),
    upcomingMeetingsList(),
  ]);
  const eventTypes = eventTypesResult.data ?? [];
  const meetings: Meeting[] = (meetingsResult.data ?? []).map(
    (meeting: UpcomingMeeting) => ({
      key: meeting.id,
      guestName: meeting.guestName,
      guestEmail: meeting.guestEmail,
      interval: formatSlotInterval(
        new Date(meeting.startDateTime),
        new Date(meeting.endDateTime),
      ),
      eventTitle: meeting.eventTitle,
    }),
  );

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col items-center gap-6 px-6 py-16">
      <h1 className="text-3xl font-semibold tracking-tight">Админка</h1>
      <AdminEventTypes initialEventTypes={eventTypes} />
      <AdminSlots eventTypes={eventTypes} />

      <section className="flex w-full max-w-2xl flex-col gap-4">
        <h2 className="text-xl font-semibold">Предстоящие встречи</h2>
        {meetings.length === 0 ? (
          <p className="text-muted-foreground">Пока нет записей</p>
        ) : (
          <ul className="flex w-full flex-col gap-3">
            {meetings.map((meeting) => (
              <li
                key={meeting.key}
                className="rounded-xl border bg-card p-6 text-card-foreground"
              >
                <p className="font-medium">{meeting.guestName}</p>
                <p className="text-muted-foreground">{meeting.guestEmail}</p>
                <p className="text-sm text-muted-foreground">
                  {meeting.eventTitle} · {meeting.interval}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
