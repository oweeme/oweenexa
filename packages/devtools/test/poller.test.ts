import { describe, expect, it, vi } from "vitest";

import { createPoller } from "../src/poller";
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

describe("createPoller", () => {
    it("fetches diagnostics on the first tick", async () => {
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());
        const onDiagnostics = vi.fn();

        const poller = createPoller({ fetchDiagnostics, onDiagnostics });
        await poller.tick();

        expect(fetchDiagnostics).toHaveBeenCalledTimes(1);
        expect(onDiagnostics).toHaveBeenCalledTimes(1);
    });

    it("does not re-fetch on a repeated tick when the path did not change", async () => {
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());
        const poller = createPoller({ fetchDiagnostics });

        await poller.tick();
        await poller.tick();

        expect(fetchDiagnostics).toHaveBeenCalledTimes(1);
    });

    it("re-fetches when force is true even if the path did not change", async () => {
        const fetchDiagnostics = vi.fn().mockResolvedValue(fakeDiagnostics());
        const poller = createPoller({ fetchDiagnostics });

        await poller.tick();
        await poller.tick(true);

        expect(fetchDiagnostics).toHaveBeenCalledTimes(2);
    });

    it("swallows a failing fetch instead of throwing", async () => {
        const fetchDiagnostics = vi.fn().mockRejectedValue(new Error("compilando"));
        const poller = createPoller({ fetchDiagnostics });

        await expect(poller.tick()).resolves.toBeUndefined();
    });
});
