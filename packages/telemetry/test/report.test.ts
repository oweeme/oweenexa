import { afterEach, describe, expect, it, vi } from "vitest";

import { defaultReportSender, type Report } from "../src/report";

const report: Report = { kind: "vital", name: "LCP", value: 1200, path: "/", timestamp: 0 };

describe("defaultReportSender", () => {
    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it("uses navigator.sendBeacon when available", () => {
        const sendBeacon = vi.fn();
        vi.stubGlobal("navigator", { sendBeacon });

        defaultReportSender("/telemetry", report);

        expect(sendBeacon).toHaveBeenCalledWith("/telemetry", JSON.stringify(report));
    });

    it("falls back to fetch with keepalive when sendBeacon is not available", () => {
        const fetchMock = vi.fn().mockResolvedValue(new Response());
        vi.stubGlobal("navigator", {});
        vi.stubGlobal("fetch", fetchMock);

        defaultReportSender("/telemetry", report);

        expect(fetchMock).toHaveBeenCalledWith("/telemetry", { method: "POST", body: JSON.stringify(report), keepalive: true });
    });

    it("does not throw when fetch rejects", async () => {
        vi.stubGlobal("navigator", {});
        vi.stubGlobal(
            "fetch",
            vi.fn().mockRejectedValue(new Error("network down")),
        );

        expect(() => defaultReportSender("/telemetry", report)).not.toThrow();
    });
});
