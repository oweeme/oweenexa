import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { initPrefetch, type PageFetcher } from "../src";

function link(href: string): HTMLAnchorElement {
    const a = document.createElement("a");
    a.href = href;
    document.body.appendChild(a);
    return a;
}

function hover(el: Element): void {
    el.dispatchEvent(new Event("pointerover", { bubbles: true }));
}

function unhover(el: Element): void {
    el.dispatchEvent(new Event("pointerout", { bubbles: true }));
}

describe("initPrefetch", () => {
    let dispose: (() => void) | undefined;

    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        dispose?.();
        dispose = undefined;
        document.body.innerHTML = "";
        vi.useRealTimers();
    });

    it("precarga un enlace interno tras el hover delay, y lo deja en la caché", async () => {
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue("<html><body>ok</body></html>");
        const cache = new Map<string, string>();
        const a = link("/about");

        dispose = initPrefetch({ fetchPage, cache, hoverDelay: 50 });

        hover(a);
        expect(fetchPage).not.toHaveBeenCalled();

        await vi.advanceTimersByTimeAsync(50);

        expect(fetchPage).toHaveBeenCalledWith("/about");
        expect(cache.get("/about")).toContain("ok");
    });

    it("un hover breve (pointerout antes del delay) no dispara el prefetch", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const a = link("/about");

        dispose = initPrefetch({ fetchPage, hoverDelay: 50 });

        hover(a);
        unhover(a);
        await vi.advanceTimersByTimeAsync(100);

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("touchstart precarga de inmediato, sin esperar ningún delay", async () => {
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue("<html><body>ok</body></html>");
        const a = link("/about");

        dispose = initPrefetch({ fetchPage, hoverDelay: 999999 });

        a.dispatchEvent(new Event("touchstart", { bubbles: true }));
        await vi.advanceTimersByTimeAsync(0);

        expect(fetchPage).toHaveBeenCalledWith("/about");
    });

    it("no precarga enlaces externos", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const a = link("https://external.example/otra");

        dispose = initPrefetch({ fetchPage, hoverDelay: 10 });
        hover(a);
        await vi.advanceTimersByTimeAsync(10);

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("no vuelve a precargar algo que ya está en caché", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const cache = new Map<string, string>([["/about", "ya en caché"]]);
        const a = link("/about");

        dispose = initPrefetch({ fetchPage, cache, hoverDelay: 10 });
        hover(a);
        await vi.advanceTimersByTimeAsync(10);

        expect(fetchPage).not.toHaveBeenCalled();
    });
});
