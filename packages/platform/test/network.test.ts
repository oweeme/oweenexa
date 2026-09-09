import { describe, expect, it, vi } from "vitest";

import { getNetworkStatus, onNetworkChange } from "../src/network";

function fakeEventTarget(): { addEventListener: ReturnType<typeof vi.fn>; removeEventListener: ReturnType<typeof vi.fn> } {
    return { addEventListener: vi.fn(), removeEventListener: vi.fn() };
}

describe("getNetworkStatus", () => {
    it("calls the Capacitor Network plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const getStatus = vi.fn().mockResolvedValue({ connected: true, connectionType: "wifi" });

        const status = await getNetworkStatus({ environment, capacitorPlugin: { getStatus, addListener: vi.fn() } });

        expect(getStatus).toHaveBeenCalled();
        expect(status).toEqual({ online: true, type: "wifi" });
    });

    it("throws a clear error when Capacitor is native but the Network plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(getNetworkStatus({ environment })).rejects.toThrow(/@capacitor\/network/);
    });

    it("uses navigator.onLine on web, with type always \"unknown\"", async () => {
        const status = await getNetworkStatus({ environment: {}, navigatorObject: { onLine: true } });
        expect(status).toEqual({ online: true, type: "unknown" });
    });

    it("reflects navigator.onLine === false", async () => {
        const status = await getNetworkStatus({ environment: {}, navigatorObject: { onLine: false } });
        expect(status.online).toBe(false);
    });
});

describe("onNetworkChange", () => {
    it("subscribes to the Capacitor networkStatusChange event and forwards status", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        let capturedListener: ((status: { connected: boolean; connectionType: string }) => void) | undefined;
        const addListener = vi.fn().mockImplementation((_event, listener) => {
            capturedListener = listener;
            return Promise.resolve({ remove });
        });
        const callback = vi.fn();

        onNetworkChange(callback, { environment, capacitorPlugin: { getStatus: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();

        capturedListener?.({ connected: false, connectionType: "none" });
        expect(callback).toHaveBeenCalledWith({ online: false, type: "none" });
    });

    it("removes the Capacitor listener when unsubscribed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        const addListener = vi.fn().mockResolvedValue({ remove });

        const unsubscribe = onNetworkChange(vi.fn(), { environment, capacitorPlugin: { getStatus: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();
        unsubscribe();
        await Promise.resolve();

        expect(remove).toHaveBeenCalled();
    });

    it("throws a clear error when Capacitor is native but the Network plugin is not installed", () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        expect(() => onNetworkChange(vi.fn(), { environment })).toThrow(/@capacitor\/network/);
    });

    it("subscribes to window online/offline events on web", () => {
        const target = fakeEventTarget();
        onNetworkChange(vi.fn(), { environment: {}, eventTarget: target });

        expect(target.addEventListener).toHaveBeenCalledWith("online", expect.any(Function));
        expect(target.addEventListener).toHaveBeenCalledWith("offline", expect.any(Function));
    });

    it("forwards online/offline web events with the right status", () => {
        const target = fakeEventTarget();
        const callback = vi.fn();
        onNetworkChange(callback, { environment: {}, eventTarget: target });

        const onlineHandler = target.addEventListener.mock.calls.find((call) => call[0] === "online")?.[1];
        const offlineHandler = target.addEventListener.mock.calls.find((call) => call[0] === "offline")?.[1];

        onlineHandler();
        expect(callback).toHaveBeenLastCalledWith({ online: true, type: "unknown" });
        offlineHandler();
        expect(callback).toHaveBeenLastCalledWith({ online: false, type: "unknown" });
    });

    it("unsubscribing on web removes both listeners", () => {
        const target = fakeEventTarget();
        const unsubscribe = onNetworkChange(vi.fn(), { environment: {}, eventTarget: target });
        unsubscribe();

        expect(target.removeEventListener).toHaveBeenCalledWith("online", expect.any(Function));
        expect(target.removeEventListener).toHaveBeenCalledWith("offline", expect.any(Function));
    });
});
