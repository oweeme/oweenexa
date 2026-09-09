import { describe, expect, it, vi } from "vitest";

import { getLaunchUrl, onOpen } from "../src/deeplinks";

describe("getLaunchUrl", () => {
    it("calls the Capacitor App plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const getLaunchUrlMock = vi.fn().mockResolvedValue({ url: "myapp://profile/42" });

        const result = await getLaunchUrl({ environment, capacitorPlugin: { getLaunchUrl: getLaunchUrlMock, addListener: vi.fn() } });

        expect(getLaunchUrlMock).toHaveBeenCalled();
        expect(result).toEqual({ url: "myapp://profile/42" });
    });

    it("normalizes a missing result from Capacitor to an empty url", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const getLaunchUrlMock = vi.fn().mockResolvedValue(undefined);

        const result = await getLaunchUrl({ environment, capacitorPlugin: { getLaunchUrl: getLaunchUrlMock, addListener: vi.fn() } });

        expect(result).toEqual({ url: "" });
    });

    it("throws a clear error when Capacitor is native but the App plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(getLaunchUrl({ environment })).rejects.toThrow(/@capacitor\/app/);
    });

    it("returns an empty url on web — same as Capacitor's own AppWeb.getLaunchUrl", async () => {
        const result = await getLaunchUrl({ environment: {} });
        expect(result).toEqual({ url: "" });
    });
});

describe("onOpen", () => {
    it("subscribes to the Capacitor appUrlOpen event and forwards it", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        let capturedListener: ((event: { url: string }) => void) | undefined;
        const addListener = vi.fn().mockImplementation((_event, listener) => {
            capturedListener = listener;
            return Promise.resolve({ remove });
        });
        const callback = vi.fn();

        onOpen(callback, { environment, capacitorPlugin: { getLaunchUrl: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();

        capturedListener?.({ url: "myapp://checkout/done" });
        expect(callback).toHaveBeenCalledWith({ url: "myapp://checkout/done" });
    });

    it("removes the Capacitor listener when unsubscribed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const remove = vi.fn().mockResolvedValue(undefined);
        const addListener = vi.fn().mockResolvedValue({ remove });

        const unsubscribe = onOpen(vi.fn(), { environment, capacitorPlugin: { getLaunchUrl: vi.fn(), addListener } });
        await Promise.resolve();
        await Promise.resolve();
        unsubscribe();
        await Promise.resolve();

        expect(remove).toHaveBeenCalled();
    });

    it("throws a clear error when Capacitor is native but the App plugin is not installed", () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        expect(() => onOpen(vi.fn(), { environment })).toThrow(/@capacitor\/app/);
    });

    it("on web, never invokes the callback — there's no equivalent event, same as Capacitor's own AppWeb", () => {
        const callback = vi.fn();
        const unsubscribe = onOpen(callback, { environment: {} });

        expect(callback).not.toHaveBeenCalled();
        expect(() => unsubscribe()).not.toThrow();
    });
});
