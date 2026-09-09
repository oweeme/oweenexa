import { afterEach, describe, expect, it, vi } from "vitest";

import { impact, notification, selectionChanged, selectionEnd, selectionStart, vibrate } from "../src/haptics";

function fakeCapacitorHapticsPlugin() {
    return {
        impact: vi.fn().mockResolvedValue(undefined),
        notification: vi.fn().mockResolvedValue(undefined),
        vibrate: vi.fn().mockResolvedValue(undefined),
        selectionStart: vi.fn().mockResolvedValue(undefined),
        selectionChanged: vi.fn().mockResolvedValue(undefined),
        selectionEnd: vi.fn().mockResolvedValue(undefined),
    };
}

function fakeNavigator(): { vibrate: ReturnType<typeof vi.fn> } {
    return { vibrate: vi.fn() };
}

describe("impact", () => {
    it("calls the real Capacitor plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorHapticsPlugin();

        await impact({ style: "MEDIUM" }, { environment, capacitorPlugin: plugin });

        expect(plugin.impact).toHaveBeenCalledWith({ style: "MEDIUM" });
    });

    it("throws a clear error when Capacitor is native but the plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(impact({}, { environment })).rejects.toThrow(/@capacitor\/haptics/);
    });

    it.each([
        ["HEAVY" as const, [61]],
        ["MEDIUM" as const, [43]],
        ["LIGHT" as const, [20]],
        [undefined, [61]],
    ])("uses the exact vibration pattern HapticsWeb uses for style=%s", async (style, expectedPattern) => {
        const nav = fakeNavigator();
        await impact({ style }, { environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).toHaveBeenCalledWith(expectedPattern);
    });

    it("throws a clear error when the Vibration API is not available", async () => {
        await expect(impact({}, { environment: {}, navigatorObject: {} })).rejects.toThrow(/Vibration API/);
    });
});

describe("notification", () => {
    it("calls the real Capacitor plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorHapticsPlugin();

        await notification({ type: "ERROR" }, { environment, capacitorPlugin: plugin });

        expect(plugin.notification).toHaveBeenCalledWith({ type: "ERROR" });
    });

    it.each([
        ["SUCCESS" as const, [35, 65, 21]],
        ["WARNING" as const, [30, 40, 30, 50, 60]],
        ["ERROR" as const, [27, 45, 50]],
        [undefined, [35, 65, 21]],
    ])("uses the exact vibration pattern HapticsWeb uses for type=%s", async (type, expectedPattern) => {
        const nav = fakeNavigator();
        await notification({ type }, { environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).toHaveBeenCalledWith(expectedPattern);
    });
});

describe("vibrate", () => {
    it("calls the real Capacitor plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorHapticsPlugin();

        await vibrate({ duration: 500 }, { environment, capacitorPlugin: plugin });

        expect(plugin.vibrate).toHaveBeenCalledWith({ duration: 500 });
    });

    it("defaults to 300ms on web, same as HapticsWeb", async () => {
        const nav = fakeNavigator();
        await vibrate({}, { environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).toHaveBeenCalledWith([300]);
    });

    it("uses the given duration on web", async () => {
        const nav = fakeNavigator();
        await vibrate({ duration: 750 }, { environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).toHaveBeenCalledWith([750]);
    });
});

describe("selectionStart / selectionChanged / selectionEnd", () => {
    afterEach(async () => {
        // El estado de selección es a nivel de módulo — se limpia entre tests.
        await selectionEnd({ environment: {} });
    });

    it("selectionChanged does nothing on web if selectionStart was never called", async () => {
        const nav = fakeNavigator();
        await selectionChanged({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).not.toHaveBeenCalled();
    });

    it("selectionChanged vibrates [70] on web after selectionStart, matching HapticsWeb", async () => {
        const nav = fakeNavigator();
        await selectionStart({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        await selectionChanged({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).toHaveBeenCalledWith([70]);
    });

    it("selectionChanged stops vibrating on web after selectionEnd", async () => {
        const nav = fakeNavigator();
        await selectionStart({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        await selectionEnd({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        await selectionChanged({ environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "vibrate"> });
        expect(nav.vibrate).not.toHaveBeenCalled();
    });

    it("calls the real Capacitor plugin for each of the three on native", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorHapticsPlugin();

        await selectionStart({ environment, capacitorPlugin: plugin });
        await selectionChanged({ environment, capacitorPlugin: plugin });
        await selectionEnd({ environment, capacitorPlugin: plugin });

        expect(plugin.selectionStart).toHaveBeenCalled();
        expect(plugin.selectionChanged).toHaveBeenCalled();
        expect(plugin.selectionEnd).toHaveBeenCalled();
    });
});
