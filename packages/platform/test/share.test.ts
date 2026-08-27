import { describe, expect, it, vi } from "vitest";

import { share } from "../src/share";

describe("share", () => {
    it("calls the Capacitor Share plugin when running natively", async () => {
        const shareFn = vi.fn().mockResolvedValue(undefined);
        const environment = { Capacitor: { isNativePlatform: () => true } };

        await share({ title: "Nexa", url: "https://example.com" }, { environment, capacitorPlugin: { share: shareFn } });

        expect(shareFn).toHaveBeenCalledWith({ title: "Nexa", url: "https://example.com" });
    });

    it("throws a clear error when Capacitor is native but the Share plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(share({ title: "x" }, { environment })).rejects.toThrow(/@capacitor\/share/);
    });

    it("uses navigator.share on web", async () => {
        const shareFn = vi.fn().mockResolvedValue(undefined);
        await share({ title: "Nexa" }, { environment: {}, navigatorObject: { share: shareFn } });

        expect(shareFn).toHaveBeenCalledWith({ title: "Nexa" });
    });

    it("throws a clear error when the Web Share API is not available", async () => {
        await expect(share({ title: "x" }, { environment: {}, navigatorObject: {} })).rejects.toThrow(/Web Share/);
    });
});
