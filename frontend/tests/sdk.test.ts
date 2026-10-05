import { describe, expect, it } from "vitest";
import {
  bookingsCreate,
  bookingsList,
  eventTypesList,
  slotsList,
  type Booking,
} from "../src/client";

describe("generated SDK", () => {
  it("exposes typed calls for all contract operations", () => {
    expect(typeof bookingsList).toBe("function");
    expect(typeof bookingsCreate).toBe("function");
    expect(typeof eventTypesList).toBe("function");
    expect(typeof slotsList).toBe("function");
  });

  it("types Booking per contract shape", () => {
    const sample: Booking = {
      id: "b1",
      slotId: "s1",
      guestName: "Гость",
      guestEmail: "g@example.com",
      createdAt: new Date().toISOString(),
    };
    expect(sample.slotId).toBe("s1");
    expect(sample.guestName).toBe("Гость");
    expect(sample.guestEmail).toBe("g@example.com");
  });
});
