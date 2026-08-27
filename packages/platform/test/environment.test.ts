import { describe, expect, it } from "vitest";

import { isCapacitor, isTauri, isWeb, capacitorPlugin } from "../src/environment";

describe("isTauri", () => {
    it("is true when the global exposes isTauri", () => {
        expect(isTauri({ isTauri: true })).toBe(true);
    });

    it("is false otherwise", () => {
        expect(isTauri({})).toBe(false);
        expect(isTauri({ isTauri: false })).toBe(false);
    });
});

describe("isCapacitor", () => {
    it("is true when Capacitor reports a native platform", () => {
        const env = { Capacitor: { isNativePlatform: () => true } };
        expect(isCapacitor(env)).toBe(true);
    });

    it("is false when Capacitor is present but running on its web target", () => {
        const env = { Capacitor: { isNativePlatform: () => false } };
        expect(isCapacitor(env)).toBe(false);
    });

    it("is false when Capacitor is not present at all", () => {
        expect(isCapacitor({})).toBe(false);
    });
});

describe("isWeb", () => {
    it("is true when neither Tauri nor native Capacitor are present", () => {
        expect(isWeb({})).toBe(true);
    });

    it("is false inside Tauri", () => {
        expect(isWeb({ isTauri: true })).toBe(false);
    });

    it("is false inside a native Capacitor app", () => {
        expect(isWeb({ Capacitor: { isNativePlatform: () => true } })).toBe(false);
    });
});

describe("capacitorPlugin", () => {
    it("returns the named plugin when it is registered", () => {
        const scheduleFn = () => Promise.resolve();
        const env = { Capacitor: { Plugins: { LocalNotifications: { schedule: scheduleFn } } } };

        const plugin = capacitorPlugin<{ schedule: typeof scheduleFn }>(env, "LocalNotifications");
        expect(plugin?.schedule).toBe(scheduleFn);
    });

    it("returns undefined when the plugin is not registered", () => {
        expect(capacitorPlugin({ Capacitor: { Plugins: {} } }, "Camera")).toBeUndefined();
    });
});
