import { afterEach, describe, expect, it } from "vitest";
import { updateActiveLinks } from "../src/active-links";

describe("updateActiveLinks — Fase 33, criterio de salida", () => {
    afterEach(() => {
        document.body.innerHTML = "";
    });

    it("marca aria-current en el link cuyo href coincide exacto con la ruta actual", () => {
        document.body.innerHTML = '<a href="/">Inicio</a><a href="/services">Servicios</a>';
        updateActiveLinks(document, "/services");

        expect(document.querySelector('a[href="/services"]')?.getAttribute("aria-current")).toBe("page");
        expect(document.querySelector('a[href="/"]')?.hasAttribute("aria-current")).toBe(false);
    });

    it("quita aria-current de un link que dejó de ser el activo", () => {
        document.body.innerHTML = '<a href="/" aria-current="page">Inicio</a><a href="/services">Servicios</a>';
        updateActiveLinks(document, "/services");

        expect(document.querySelector('a[href="/"]')?.hasAttribute("aria-current")).toBe(false);
        expect(document.querySelector('a[href="/services"]')?.getAttribute("aria-current")).toBe("page");
    });

    it('respeta data-nexa-match="prefix" para coincidencia por subruta', () => {
        document.body.innerHTML = '<a href="/dashboard" data-nexa-match="prefix">Panel</a>';
        updateActiveLinks(document, "/dashboard/settings");

        expect(document.querySelector("a")?.getAttribute("aria-current")).toBe("page");
    });

    it('href="/" en modo prefijo no matchea cualquier ruta', () => {
        document.body.innerHTML = '<a href="/" data-nexa-match="prefix">Inicio</a>';
        updateActiveLinks(document, "/services");

        expect(document.querySelector("a")?.hasAttribute("aria-current")).toBe(false);
    });

    it("recalcula solo dentro de root cuando se le pasa un elemento en vez de document", () => {
        document.body.innerHTML =
            '<nav><a href="/services">Header</a></nav><div id="slot"><a href="/services">Slot</a></div>';
        const slot = document.getElementById("slot")!;

        updateActiveLinks(slot, "/services");

        expect(document.querySelector("nav a")?.hasAttribute("aria-current")).toBe(false);
        expect(slot.querySelector("a")?.getAttribute("aria-current")).toBe("page");
    });
});
