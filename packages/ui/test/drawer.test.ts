import { describe, expect, it } from "vitest";
import { closeDrawer, openDrawer } from "../src";

function setupDrawer(): { trigger: HTMLButtonElement; drawer: HTMLDivElement; first: HTMLButtonElement; last: HTMLButtonElement } {
    document.body.innerHTML = "";

    const trigger = document.createElement("button");
    trigger.textContent = "Abrir menú";
    document.body.appendChild(trigger);

    const drawer = document.createElement("div");
    drawer.className = "nx-drawer";
    drawer.setAttribute("hidden", "");
    document.body.appendChild(drawer);

    const first = document.createElement("button");
    first.textContent = "Primero";
    const last = document.createElement("button");
    last.textContent = "Último";
    drawer.append(first, last);

    trigger.focus();

    return { trigger, drawer, first, last };
}

describe("openDrawer / closeDrawer", () => {
    it("quita `hidden` y pone los atributos ARIA de un diálogo modal", () => {
        const { drawer } = setupDrawer();
        openDrawer(drawer);

        expect(drawer.hasAttribute("hidden")).toBe(false);
        expect(drawer.getAttribute("role")).toBe("dialog");
        expect(drawer.getAttribute("aria-modal")).toBe("true");
    });

    it("mueve el foco dentro del drawer al abrir", () => {
        const { drawer, first } = setupDrawer();
        openDrawer(drawer);

        expect(document.activeElement).toBe(first);
    });

    it("devuelve el foco a quien abrió el drawer, al cerrarlo", () => {
        const { trigger, drawer } = setupDrawer();
        openDrawer(drawer);
        closeDrawer(drawer);

        expect(drawer.hasAttribute("hidden")).toBe(true);
        expect(document.activeElement).toBe(trigger);
    });

    it("Escape cierra el drawer", () => {
        const { drawer } = setupDrawer();
        openDrawer(drawer);

        drawer.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

        expect(drawer.hasAttribute("hidden")).toBe(true);
    });

    it("Tab en el último elemento enfocable vuelve al primero (focus trap)", () => {
        const { drawer, first, last } = setupDrawer();
        openDrawer(drawer);
        last.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
        drawer.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(first);
    });

    it("Shift+Tab en el primer elemento enfocable va al último (focus trap)", () => {
        const { drawer, first, last } = setupDrawer();
        openDrawer(drawer);
        first.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true });
        drawer.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(last);
    });

    it("closeDrawer es seguro de llamar sobre un drawer ya cerrado", () => {
        const { drawer } = setupDrawer();
        expect(() => closeDrawer(drawer)).not.toThrow();
        expect(drawer.hasAttribute("hidden")).toBe(true);
    });

    it("closeDrawer es seguro de llamar sobre un drawer desconectado del DOM (nodo reemplazado por una navegación SPA)", () => {
        const { drawer } = setupDrawer();
        openDrawer(drawer);
        drawer.remove();

        expect(() => closeDrawer(drawer)).not.toThrow();
    });
});
