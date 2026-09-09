import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
});

afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
});

describe("theme.get()", () => {
    it("devuelve \"system\" cuando nunca se guardó nada", async () => {
        const { theme } = await import("../src/theme");
        expect(theme.get()).toBe("system");
    });

    it("devuelve el valor guardado por set()", async () => {
        const { theme } = await import("../src/theme");
        theme.set("dark");
        expect(theme.get()).toBe("dark");
    });
});

describe("theme.set()", () => {
    it("pone data-theme=\"dark\" en <html>", async () => {
        const { theme } = await import("../src/theme");
        theme.set("dark");
        expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    });

    it("pone data-theme=\"light\" en <html>", async () => {
        const { theme } = await import("../src/theme");
        theme.set("light");
        expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    });

    it("\"system\" quita el atributo data-theme (vuelve a prefers-color-scheme)", async () => {
        const { theme } = await import("../src/theme");
        theme.set("dark");
        theme.set("system");
        expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    });

    it("persiste en localStorage real, no solo en memoria", async () => {
        const { theme } = await import("../src/theme");
        theme.set("dark");
        expect(localStorage.getItem("nexa-theme")).toBe("dark");
    });
});

describe("persistencia entre \"recargas\" (reimportación del módulo)", () => {
    it("una preferencia guardada se reaplica apenas el módulo se evalúa de nuevo", async () => {
        localStorage.setItem("nexa-theme", "dark");

        vi.resetModules();
        await import("../src/theme");

        expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    });

    it("sin nada guardado, reimportar el módulo no toca data-theme", async () => {
        vi.resetModules();
        await import("../src/theme");

        expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    });
});
