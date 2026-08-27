import { beforeEach, describe, expect, it } from "vitest";

import { swapDocument } from "../src/swap";

describe("swapDocument", () => {
    beforeEach(() => {
        document.title = "vieja";
        document.body.innerHTML = "<p>contenido viejo</p>";
    });

    it("replaces the title and the body content", () => {
        swapDocument(document, "<html><head><title>nueva</title></head><body><p>contenido nuevo</p></body></html>");

        expect(document.title).toBe("nueva");
        expect(document.body.innerHTML).toContain("contenido nuevo");
        expect(document.body.innerHTML).not.toContain("contenido viejo");
    });

    it("re-creates <script> tags instead of leaving the inert copy that innerHTML would produce", () => {
        const before = document.body.querySelector("script");
        expect(before).toBeNull();

        swapDocument(
            document,
            '<html><body><p>hola</p><script type="module" data-marker="boot">window.__marker = 1;</script></body></html>',
        );

        const scripts = document.body.querySelectorAll("script");
        expect(scripts).toHaveLength(1);
        expect(scripts[0]?.getAttribute("type")).toBe("module");
        expect(scripts[0]?.getAttribute("data-marker")).toBe("boot");
        expect(scripts[0]?.textContent).toBe("window.__marker = 1;");
    });

    it("restores the scroll position after the swap", () => {
        window.scrollTo(0, 500);

        swapDocument(document, "<html><body><p>contenido largo</p></body></html>");

        expect(window.scrollX).toBe(0);
        expect(window.scrollY).toBe(500);
    });
});
