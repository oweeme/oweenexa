import { describe, expect, it } from "vitest";

import { observeVitals, type VitalReport } from "../src/vitals";

/** Un `PerformanceObserver` falso: guarda el callback registrado para
 * cada tipo y deja que el test lo dispare a mano con entradas falsas —
 * un DOM de pruebas nunca produce estas entradas de rendimiento de
 * verdad. */
function fakeObserverClass(supportedEntryTypes: string[]) {
    const registered: Array<{ type: string; fire: (entries: unknown[]) => void }> = [];

    class FakePerformanceObserver {
        static supportedEntryTypes = supportedEntryTypes;
        #callback: (list: { getEntries: () => unknown[] }) => void;

        constructor(callback: (list: { getEntries: () => unknown[] }) => void) {
            this.#callback = callback;
        }

        observe(options: { type: string }) {
            registered.push({
                type: options.type,
                fire: (entries) => this.#callback({ getEntries: () => entries }),
            });
        }

        disconnect() {}
    }

    return { FakePerformanceObserver, registered };
}

describe("observeVitals", () => {
    it("reports LCP from the last largest-contentful-paint entry", () => {
        const { FakePerformanceObserver, registered } = fakeObserverClass(["largest-contentful-paint"]);
        const reports: VitalReport[] = [];

        observeVitals((r) => reports.push(r), { PerformanceObserverCtor: FakePerformanceObserver as never });

        const lcpEntry = registered.find((r) => r.type === "largest-contentful-paint")!;
        lcpEntry.fire([{ renderTime: 1500, startTime: 1500 }]);

        expect(reports).toEqual([{ name: "LCP", value: 1500 }]);
    });

    it("accumulates CLS across multiple layout-shift entries, ignoring ones with recent input", () => {
        const { FakePerformanceObserver, registered } = fakeObserverClass(["layout-shift"]);
        const reports: VitalReport[] = [];

        observeVitals((r) => reports.push(r), { PerformanceObserverCtor: FakePerformanceObserver as never });

        const cls = registered.find((r) => r.type === "layout-shift")!;
        cls.fire([{ value: 0.05, hadRecentInput: false }]);
        cls.fire([{ value: 0.2, hadRecentInput: true }, { value: 0.03, hadRecentInput: false }]);

        expect(reports.map((r) => r.value)).toEqual([0.05, 0.08]);
    });

    it("does not observe an entry type the browser does not support", () => {
        const { FakePerformanceObserver, registered } = fakeObserverClass(["layout-shift"]);

        observeVitals(() => {}, { PerformanceObserverCtor: FakePerformanceObserver as never });

        expect(registered.some((r) => r.type === "largest-contentful-paint")).toBe(false);
    });

    it("returns a working disconnect function", () => {
        const { FakePerformanceObserver } = fakeObserverClass(["layout-shift"]);
        const stop = observeVitals(() => {}, { PerformanceObserverCtor: FakePerformanceObserver as never });

        expect(() => stop()).not.toThrow();
    });
});
