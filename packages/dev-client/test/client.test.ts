import { afterEach, describe, expect, it, vi } from "vitest";

import { initDevClient } from "../src/client";

describe("initDevClient", () => {
    afterEach(() => {
        vi.useRealTimers();
    });

    it("schedules a repeating poll and ticks once immediately", async () => {
        vi.useFakeTimers();
        const fetchVersion = vi.fn().mockResolvedValue("v1");

        const stop = initDevClient({ fetchVersion, intervalMs: 1000 });
        // Un avance de 0ms no dispara el intervalo (programado a
        // 1000ms), pero sí deja correr la llamada inmediata (que va por
        // el microtask queue, no por el reloj falso).
        await vi.advanceTimersByTimeAsync(0);
        const afterImmediateTick = fetchVersion.mock.calls.length;
        expect(afterImmediateTick).toBeGreaterThanOrEqual(1);

        await vi.advanceTimersByTimeAsync(1000);
        expect(fetchVersion.mock.calls.length).toBe(afterImmediateTick + 1);

        stop();
        await vi.advanceTimersByTimeAsync(2000);
        expect(fetchVersion.mock.calls.length).toBe(afterImmediateTick + 1);
    });
});
