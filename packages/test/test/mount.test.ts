import { afterEach, describe, expect, it } from "vitest";

import { mountChunk } from "../src/mount";

describe("mountChunk", () => {
    afterEach(() => {
        document.body.innerHTML = "";
    });

    it("mounts the HTML into the document and calls activate with the root element", () => {
        let received: Element | null = null;
        const activate = (el: Element) => {
            received = el;
        };

        const mounted = mountChunk(activate, `<button>Comprar</button>`);

        expect(mounted.el.tagName).toBe("BUTTON");
        expect(mounted.el.isConnected).toBe(true);
        expect(received).toBe(mounted.el);
    });

    it("lets the test fire real events and observe the handler's effect", () => {
        const activate = (el: Element) => {
            el.addEventListener("click", () => {
                el.textContent = "Comprado";
            });
        };

        const mounted = mountChunk(activate, `<button>Comprar</button>`);
        mounted.el.click();

        expect(mounted.el.textContent).toBe("Comprado");
    });

    it("destroy removes the mounted element from the document", () => {
        const mounted = mountChunk(() => {}, `<button>x</button>`);
        expect(mounted.el.isConnected).toBe(true);

        mounted.destroy();
        expect(mounted.el.isConnected).toBe(false);
    });

    it("throws a clear error when the HTML has no single root element", () => {
        expect(() => mountChunk(() => {}, `<span>a</span><span>b</span>`)).toThrow(/único elemento raíz/);
    });
});
