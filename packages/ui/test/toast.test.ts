import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { notify } from "../src";

beforeEach(() => {
    document.body.innerHTML = "";
    document.head.querySelectorAll("style[data-nexa-ui-toast]").forEach((el) => el.remove());
    vi.useFakeTimers();
});

afterEach(() => {
    vi.useRealTimers();
});

describe("notify", () => {
    it("crea un toast real en el DOM, con el mensaje y la variante pedidos", () => {
        notify({ message: "Guardado con éxito", variant: "success" });

        const toast = document.querySelector(".nx-toast");
        expect(toast).not.toBeNull();
        expect(toast?.classList.contains("nx-toast-success")).toBe(true);
        expect(toast?.textContent).toContain("Guardado con éxito");
    });

    it("es accesible: role=status y aria-live=polite", () => {
        notify({ message: "Hola" });
        const toast = document.querySelector(".nx-toast");
        expect(toast?.getAttribute("role")).toBe("status");
        expect(toast?.getAttribute("aria-live")).toBe("polite");
    });

    it("dos toasts se apilan — no se reemplazan entre sí", () => {
        notify({ message: "Primero" });
        notify({ message: "Segundo" });
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(2);
    });

    it("se auto-cierra después de `duration` (default 4000ms)", () => {
        notify({ message: "Chau en 4s" });
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(1);

        vi.advanceTimersByTime(4000);
        vi.advanceTimersByTime(300); // la transición de salida
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(0);
    });

    it("duration: 0 desactiva el auto-cierre", () => {
        notify({ message: "Quedate", duration: 0 });
        vi.advanceTimersByTime(60_000);
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(1);
    });

    it("el botón de cerrar lo saca del DOM antes de que termine `duration`", () => {
        notify({ message: "Cerrame" });
        const closeBtn = document.querySelector<HTMLButtonElement>(".nx-toast-close");
        closeBtn?.click();

        vi.advanceTimersByTime(300);
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(0);
    });

    it("close() del handle devuelto cierra el toast a mano", () => {
        const handle = notify({ message: "Cerrame por API", duration: 0 });
        handle.close();

        vi.advanceTimersByTime(300);
        expect(document.querySelectorAll(".nx-toast")).toHaveLength(0);
    });

    it("solo inyecta el <style> una vez, sin importar cuántos toasts se creen", () => {
        notify({ message: "Uno" });
        notify({ message: "Dos" });
        notify({ message: "Tres" });
        expect(document.head.querySelectorAll("style[data-nexa-ui-toast]")).toHaveLength(1);
    });
});
