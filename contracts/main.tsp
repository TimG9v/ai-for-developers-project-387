import "@typespec/http";
import "@typespec/rest";

using TypeSpec.Http;
using TypeSpec.Rest;

@service(#{
  title: "Calendar — API записи на звонок",
})
namespace Booking {
  @doc("Тип встречи: разновидность встречи, которую задаёт владелец (название, описание, длительность).")
  model EventType {
    id: string;
    title: string;
    @doc("Описание типа встречи")
    description?: string;
    @doc("Длительность в минутах; определяет длительность слотов этого типа")
    durationMinutes: int32;
  }

  @doc("Слот: свободный интервал, опубликованный владельцем; длительность определяется типом встречи.")
  model Slot {
    id: string;
    @doc("Тип встречи, к которому относится слот")
    eventTypeId: string;
    startDateTime: utcDateTime;
    endDateTime: utcDateTime;
  }

  @doc("Запись: слот, закреплённый за гостем.")
  model Booking {
    id: string;
    slotId: string;
    @doc("Имя гостя — обязательно")
    guestName: string;
    @doc("Email гостя — обязательно")
    guestEmail: string;
    createdAt: utcDateTime;
  }

  @doc("Предстоящая встреча: ракурс владельца — запись со временем слота и типом встречи.")
  model UpcomingMeeting {
    id: string;
    slotId: string;
    guestName: string;
    guestEmail: string;
    startDateTime: utcDateTime;
    endDateTime: utcDateTime;
    eventTypeId: string;
    eventTitle: string;
  }

  @doc("Слот уже занят: на слот не может быть более одной записи.")
  model ConflictError {
    @statusCode statusCode: 409;
  }

  @doc("Запись, слот или тип встречи не найдены.")
  model NotFoundError {
    @statusCode statusCode: 404;
  }

  @doc("Запрос переноса записи: новый свободный слот того же типа встречи.")
  model RescheduleRequest {
    @doc("Новый слот — свободный, того же типа встречи")
    newSlotId: string;
  }

  @doc("Невалидный ввод: обязательные поля пусты или значения вне допустимых границ.")
  model ValidationError {
    @statusCode statusCode: 400;
  }

  @doc("Типы встреч: создаёт владелец, смотрят все.")
  @route("/event-types")
  interface EventTypes {
    @get list(): EventType[];
    @post create(@body body: EventType): EventType | ValidationError;
  }

  @doc("Слоты: публикует владелец. Свободные слоты выдаются в окне 14 дней от текущей даты.")
  @route("/slots")
  interface Slots {
    @get list(@query eventTypeId?: string): Slot[];
    @post create(@body body: Slot): Slot | ValidationError | NotFoundError;
  }

  @doc("""
    Записи: список — ракурс владельца (все типы встреч); создание — действие
    гостя. Отмена и перенос доступны гостю по ссылке управления записью
    (id записи из подтверждения — фактически unguessable-токен, АDR 0002) и
    владельцу из админки. Отмена освобождает слот; перенос — на свободный
    слот того же типа встречи.
  """)
  @route("/bookings")
  interface Bookings {
    @get list(): Booking[];
    @post create(@body body: Booking): Booking | ConflictError | NotFoundError | ValidationError;

    @doc("Отмена записи: слот освобождается и снова появляется в календаре.")
    @delete cancel(@path id: string): void | NotFoundError;

    @doc("Перенос записи на другой свободный слот того же типа встречи.")
    @post
    @route("/{id}/reschedule")
    reschedule(@path id: string, @body body: RescheduleRequest): Booking | NotFoundError | ConflictError | ValidationError;
  }

  @doc("Предстоящие встречи: записи со временем слота и типом встречи; прошедшие не отдаются.")
  @route("/upcoming-meetings")
  interface UpcomingMeetings {
    @get list(): UpcomingMeeting[];
  }
}
