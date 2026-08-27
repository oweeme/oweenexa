import { describe, expect, it } from "vitest";

import { observeErrors } from "../src/errors";

describe("observeErrors", () => {
    it("reports a message for a window error event, including file/line when present", () => {
        const reports: Array<{ name: string; message: string }> = [];
        const stop = observeErrors((r) => reports.push(r), { window });

        window.dispatchEvent(
            new ErrorEvent("error", { message: "boom", filename: "app.js", lineno: 12, colno: 3 }),
        );

        expect(reports).toHaveLength(1);
        expect(reports[0].name).toBe("error");
        expect(reports[0].message).toBe("boom (app.js:12:3)");

        stop();
    });

    it("reports an unhandled promise rejection with the Error's message", () => {
        const reports: Array<{ name: string; message: string }> = [];
        const stop = observeErrors((r) => reports.push(r), { window });

        const event = new Event("unhandledrejection") as PromiseRejectionEvent & { reason: unknown };
        Object.defineProperty(event, "reason", { value: new Error("fetch failed") });
        window.dispatchEvent(event);

        expect(reports).toEqual([{ name: "unhandledrejection", message: "fetch failed" }]);

        stop();
    });

    it("stops reporting after the returned function is called", () => {
        const reports: unknown[] = [];
        const stop = observeErrors((r) => reports.push(r), { window });
        stop();

        window.dispatchEvent(new ErrorEvent("error", { message: "boom" }));

        expect(reports).toHaveLength(0);
    });
});
