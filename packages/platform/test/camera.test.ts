import { describe, expect, it, vi } from "vitest";

import { capturePhoto } from "../src/camera";

describe("capturePhoto", () => {
    it("uses the Capacitor Camera plugin when running natively", async () => {
        const getPhoto = vi.fn().mockResolvedValue({ dataUrl: "data:image/png;base64,abc" });
        const environment = { Capacitor: { isNativePlatform: () => true } };

        const result = await capturePhoto({ environment, capacitorPlugin: { getPhoto } });

        expect(result.dataUrl).toBe("data:image/png;base64,abc");
        expect(getPhoto).toHaveBeenCalledWith({ resultType: "dataUrl", quality: 90 });
    });

    it("throws a clear error when Capacitor is native but the Camera plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(capturePhoto({ environment })).rejects.toThrow(/@capacitor\/camera/);
    });

    it("throws when the native plugin resolves without a dataUrl", async () => {
        const getPhoto = vi.fn().mockResolvedValue({});
        const environment = { Capacitor: { isNativePlatform: () => true } };

        await expect(capturePhoto({ environment, capacitorPlugin: { getPhoto } })).rejects.toThrow(/ninguna imagen/);
    });

    it("delegates to the injected web capture on web/Tauri", async () => {
        const webCapture = vi.fn().mockResolvedValue({ dataUrl: "data:image/png;base64,xyz" });

        const result = await capturePhoto({ environment: {}, webCapture });

        expect(result.dataUrl).toBe("data:image/png;base64,xyz");
        expect(webCapture).toHaveBeenCalledTimes(1);
    });
});
