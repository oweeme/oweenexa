import { describe, expect, it, vi } from "vitest";

import { createPoller } from "../src/poller";

describe("createPoller", () => {
    it("the first tick only establishes a baseline, without reloading", async () => {
        const fetchVersion = vi.fn().mockResolvedValue("v1");
        const fetchPage = vi.fn();

        const poller = createPoller({ fetchVersion, fetchPage });
        await poller.tick();

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("a repeated tick with the same version does not reload the page", async () => {
        const fetchVersion = vi.fn().mockResolvedValue("v1");
        const fetchPage = vi.fn();

        const poller = createPoller({ fetchVersion, fetchPage });
        await poller.tick();
        await poller.tick();
        await poller.tick();

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("fetches and swaps the current page once the version changes", async () => {
        const fetchVersion = vi.fn().mockResolvedValueOnce("v1").mockResolvedValueOnce("v2");
        const fetchPage = vi.fn().mockResolvedValue("<html><head><title>nueva</title></head><body>nuevo</body></html>");
        document.body.innerHTML = "<p>viejo</p>";
        history.pushState({}, "", "/products/iphone-17");

        const poller = createPoller({ fetchVersion, fetchPage });
        await poller.tick();
        await poller.tick();

        expect(fetchPage).toHaveBeenCalledWith("/products/iphone-17");
        expect(document.title).toBe("nueva");
        expect(document.body.textContent).toBe("nuevo");
    });

    it("adopts the new version as the baseline, so a third identical tick does not reload again", async () => {
        const fetchVersion = vi.fn().mockResolvedValueOnce("v1").mockResolvedValueOnce("v2").mockResolvedValueOnce("v2");
        const fetchPage = vi.fn().mockResolvedValue("<html><body>nuevo</body></html>");

        const poller = createPoller({ fetchVersion, fetchPage });
        await poller.tick();
        await poller.tick();
        await poller.tick();

        expect(fetchPage).toHaveBeenCalledTimes(1);
    });

    it("swallows a failing fetch instead of throwing, so the next tick can still recover", async () => {
        const fetchVersion = vi.fn().mockRejectedValue(new Error("servidor caído un instante"));

        const poller = createPoller({ fetchVersion, fetchPage: vi.fn() });

        await expect(poller.tick()).resolves.toBeUndefined();
    });
});
