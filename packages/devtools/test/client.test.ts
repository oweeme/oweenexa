import { afterEach, describe, expect, it, vi } from "vitest";

import { initDevtools } from "../src/client";
import type { Diagnostics } from "../src/diagnostics";

function fakeDiagnostics(): Diagnostics {
    return {
        pattern: "/",
        classification: { static: 1, dynamic: 0, interactive: 0, async: 0, total: 1 },
        initialJsBytes: 100,
        activation: [],
        seoWarnings: [],
        pkgWarnings: [],
    };
}

describe("initDevtools", () => {
    afterEach(() => {
        vi.useRealTimers();
        document.body.innerHTML = "";
    });

    it("creates the panel on the first tick", async () => {
        vi.useFakeTimers();
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());

        const stop = initDevtools({ fetchDiagnostics, intervalMs: 1000 });
        await vi.advanceTimersByTimeAsync(0);

        expect(document.querySelector("[data-nexa-devtools]")).not.toBeNull();
        stop();
    });

    it("recreates the panel after its container is removed from the DOM, even on the same path", async () => {
        vi.useFakeTimers();
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());

        const stop = initDevtools({ fetchDiagnostics, intervalMs: 1000 });
        await vi.advanceTimersByTimeAsync(0);
        expect(fetchDiagnostics).toHaveBeenCalledTimes(1);

        // Simula lo que hace `@nexa/dev-client` en un auto-reload: borra
        // todo `<body>`.
        document.body.innerHTML = "";
        expect(document.querySelector("[data-nexa-devtools]")).toBeNull();

        await vi.advanceTimersByTimeAsync(1000);

        expect(fetchDiagnostics).toHaveBeenCalledTimes(2);
        expect(document.querySelector("[data-nexa-devtools]")).not.toBeNull();
        stop();
    });

    it("stops polling once stopped", async () => {
        vi.useFakeTimers();
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());

        const stop = initDevtools({ fetchDiagnostics, intervalMs: 1000 });
        await vi.advanceTimersByTimeAsync(0);
        stop();

        await vi.advanceTimersByTimeAsync(5000);
        expect(fetchDiagnostics).toHaveBeenCalledTimes(1);
    });
});
