import { describe, expect, it, vi } from "vitest";

import { getLifecycleState, onLifecycleChange } from "../src/lifecycle";

function fakeDocument(hidden: boolean): { hidden: boolean; addEventListener: ReturnType<typeof vi.fn>; removeEventListener: ReturnType<typeof vi.fn> } {
    return { hidden, addEventListener: vi.fn(), removeEventListener: vi.fn() };
}

describe("getLifecycleState", () => {
    it("calls the Capacitor App plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const getState = vi.fn().mockResolvedValue({ isActive: true });

        const state = await getLifecycleState({ environment, capacitorPlugin: { getState, addListener: vi.fn() } });

        expect(getState).toHaveBeenCalled();
        expect(state).toEqual({ active: true });
    });

    it("throws a clear error when Capacitor is native but the App plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(getLifecycleState({ environment })).rejects.toThrow(/@capacitor\/app/);
    });

    it("uses document.hidden on web: visible tab is active", async () => {
        const state = await getLifecycleState({ environment: {}, documentObject: fakeDocument(false) });
        expect(state).toEqual({ active: true });
    });

    it("uses document.hidden on web: hidden tab is not active", async () => {
        const state = await getLifecycleState({ environment: {}, documentObject: fakeDocument(true) });
        expect(state).toEqual({ active: false });
    });
});

describe("onLifecycleChange", () => {
    it("subscribes to the Capacitor appStateChange event and forwards state", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        let capturedListener: ((state: { isActive: boolean }) => void) | undefined;
        const addListener = vi.fn().mockImplementation((_event, listener) => {
            capturedListener = listener;
            return Promise.resolve({ remove });
        });
        const callback = vi.fn();

        onLifecycleChange(callback, { environment, capacitorPlugin: { getState: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();

        capturedListener?.({ isActive: false });
        expect(callback).toHaveBeenCalledWith({ active: false });
    });

    it("removes the Capacitor listener when unsubscribed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        const addListener = vi.fn().mockResolvedValue({ remove });

        const unsubscribe = onLifecycleChange(vi.fn(), { environment, capacitorPlugin: { getState: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();
        unsubscribe();
        await Promise.resolve();

        expect(remove).toHaveBeenCalled();
    });

    it("throws a clear error when Capacitor is native but the App plugin is not installed", () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        expect(() => onLifecycleChange(vi.fn(), { environment })).toThrow(/@capacitor\/app/);
    });

    it("subscribes to document visibilitychange on web", () => {
        const doc = fakeDocument(false);
        onLifecycleChange(vi.fn(), { environment: {}, documentObject: doc });

        expect(doc.addEventListener).toHaveBeenCalledWith("visibilitychange", expect.any(Function));
    });

    it("forwards visibilitychange with the current document.hidden value", () => {
        const doc = fakeDocument(false);
        const callback = vi.fn();
        onLifecycleChange(callback, { environment: {}, documentObject: doc });

        const handler = doc.addEventListener.mock.calls[0][1];
        doc.hidden = true;
        handler();
        expect(callback).toHaveBeenCalledWith({ active: false });

        doc.hidden = false;
        handler();
        expect(callback).toHaveBeenCalledWith({ active: true });
    });

    it("unsubscribing on web removes the visibilitychange listener", () => {
        const doc = fakeDocument(false);
        const unsubscribe = onLifecycleChange(vi.fn(), { environment: {}, documentObject: doc });
        unsubscribe();

        expect(doc.removeEventListener).toHaveBeenCalledWith("visibilitychange", expect.any(Function));
    });
});
