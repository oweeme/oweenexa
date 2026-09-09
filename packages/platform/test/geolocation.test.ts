import { describe, expect, it, vi } from "vitest";

import { getCurrentPosition, watchPosition } from "../src/geolocation";

function fakePosition(latitude: number, longitude: number) {
    return {
        coords: {
            latitude,
            longitude,
            accuracy: 10,
            altitude: null,
            altitudeAccuracy: null,
            heading: null,
            speed: null,
        },
        timestamp: 1700000000000,
    };
}

function fakeNavigatorGeolocation(): { getCurrentPosition: ReturnType<typeof vi.fn>; watchPosition: ReturnType<typeof vi.fn>; clearWatch: ReturnType<typeof vi.fn> } {
    return { getCurrentPosition: vi.fn(), watchPosition: vi.fn(), clearWatch: vi.fn() };
}

describe("getCurrentPosition", () => {
    it("calls the Capacitor Geolocation plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const position = fakePosition(-34.6, -58.4);
        const getCurrentPositionMock = vi.fn().mockResolvedValue(position);

        const result = await getCurrentPosition(
            { enableHighAccuracy: true },
            { environment, capacitorPlugin: { getCurrentPosition: getCurrentPositionMock, watchPosition: vi.fn(), clearWatch: vi.fn() } },
        );

        expect(getCurrentPositionMock).toHaveBeenCalledWith({ enableHighAccuracy: true });
        expect(result).toEqual(position);
    });

    it("throws a clear error when Capacitor is native but the Geolocation plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(getCurrentPosition({}, { environment })).rejects.toThrow(/@capacitor\/geolocation/);
    });

    it("uses navigator.geolocation on web", async () => {
        const geo = fakeNavigatorGeolocation();
        const position = fakePosition(40.7, -74.0);
        geo.getCurrentPosition.mockImplementation((success: (p: unknown) => void) => success(position));

        const result = await getCurrentPosition({}, { environment: {}, navigatorObject: { geolocation: geo as unknown as Geolocation } });

        expect(result).toEqual(position);
    });

    it("rejects with a clear error when the browser's Geolocation API reports an error", async () => {
        const geo = fakeNavigatorGeolocation();
        geo.getCurrentPosition.mockImplementation((_success: unknown, error: (e: { message: string }) => void) =>
            error({ message: "User denied Geolocation" }),
        );

        await expect(
            getCurrentPosition({}, { environment: {}, navigatorObject: { geolocation: geo as unknown as Geolocation } }),
        ).rejects.toThrow(/User denied Geolocation/);
    });

    it("throws a clear error when the Geolocation API is not available", async () => {
        await expect(getCurrentPosition({}, { environment: {}, navigatorObject: {} })).rejects.toThrow(/Geolocation/);
    });
});

describe("watchPosition", () => {
    it("subscribes to the Capacitor watchPosition callback and forwards positions", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const clearWatch = vi.fn().mockResolvedValue(undefined);
        let capturedCallback: ((position: unknown) => void) | undefined;
        const watchPositionMock = vi.fn().mockImplementation((_options, callback) => {
            capturedCallback = callback;
            return Promise.resolve("watch-1");
        });
        const callback = vi.fn();

        watchPosition(callback, {}, { environment, capacitorPlugin: { getCurrentPosition: vi.fn(), watchPosition: watchPositionMock, clearWatch } });
        await Promise.resolve();
        await Promise.resolve();

        const position = fakePosition(1, 2);
        capturedCallback?.(position);
        expect(callback).toHaveBeenCalledWith(position);
    });

    it("clears the Capacitor watch when unsubscribed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const clearWatch = vi.fn().mockResolvedValue(undefined);
        const watchPositionMock = vi.fn().mockResolvedValue("watch-1");

        const unsubscribe = watchPosition(vi.fn(), {}, { environment, capacitorPlugin: { getCurrentPosition: vi.fn(), watchPosition: watchPositionMock, clearWatch } });
        await Promise.resolve();
        await Promise.resolve();
        unsubscribe();

        expect(clearWatch).toHaveBeenCalledWith({ id: "watch-1" });
    });

    it("throws a clear error when Capacitor is native but the Geolocation plugin is not installed", () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        expect(() => watchPosition(vi.fn(), {}, { environment })).toThrow(/@capacitor\/geolocation/);
    });

    it("subscribes to navigator.geolocation.watchPosition on web and forwards positions", () => {
        const geo = fakeNavigatorGeolocation();
        geo.watchPosition.mockReturnValue(42);
        const callback = vi.fn();

        watchPosition(callback, {}, { environment: {}, navigatorObject: { geolocation: geo as unknown as Geolocation } });

        const position = fakePosition(5, 6);
        const successHandler = geo.watchPosition.mock.calls[0][0];
        successHandler(position);
        expect(callback).toHaveBeenCalledWith(position);
    });

    it("unsubscribing on web calls navigator.geolocation.clearWatch with the right id", () => {
        const geo = fakeNavigatorGeolocation();
        geo.watchPosition.mockReturnValue(42);

        const unsubscribe = watchPosition(vi.fn(), {}, { environment: {}, navigatorObject: { geolocation: geo as unknown as Geolocation } });
        unsubscribe();

        expect(geo.clearWatch).toHaveBeenCalledWith(42);
    });
});
