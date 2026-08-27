import { describe, expect, it } from "vitest";

import { initTelemetry } from "../src/client";
import type { Report } from "../src/report";

describe("initTelemetry", () => {
    it("sends an error report to the configured endpoint via sendReport", () => {
        const sent: Array<[string, Report]> = [];
        const stop = initTelemetry({
            endpoint: "/telemetry",
            sendReport: (endpoint, report) => sent.push([endpoint, report]),
            window,
        });

        window.dispatchEvent(new ErrorEvent("error", { message: "boom" }));

        expect(sent).toHaveLength(1);
        expect(sent[0][0]).toBe("/telemetry");
        expect(sent[0][1]).toMatchObject({ kind: "error", name: "error", detail: "boom", path: "/" });

        stop();
    });

    it("also calls onReport for every emitted report", () => {
        const received: Report[] = [];
        const stop = initTelemetry({
            endpoint: "/telemetry",
            sendReport: () => {},
            onReport: (r) => received.push(r),
            window,
        });

        window.dispatchEvent(new ErrorEvent("error", { message: "boom" }));

        expect(received).toHaveLength(1);
        stop();
    });

    it("stop() disconnects the error listeners", () => {
        const sent: Report[] = [];
        const stop = initTelemetry({ endpoint: "/telemetry", sendReport: (_e, r) => sent.push(r), window });
        stop();

        window.dispatchEvent(new ErrorEvent("error", { message: "boom" }));

        expect(sent).toHaveLength(0);
    });
});
