import { describe, expect, it, vi } from "vitest";
import { initActivation, type ActivationManifest, type ModuleLoader } from "../src";

function makeEl(id: string, tag = "button"): HTMLElement {
    const el = document.createElement(tag);
    el.setAttribute("data-nexa", id);
    document.body.appendChild(el);
    return el;
}

describe("initActivation — Fase 5, criterio de salida", () => {
    it("un manifiesto vacío (página 100% estática) nunca llama a loadModule", () => {
        const loadModule = vi.fn<ModuleLoader>();
        const manifest: ActivationManifest = {};

        initActivation(manifest, { loadModule });

        expect(loadModule).not.toHaveBeenCalled();
    });

    it("estrategia 'interaction': no carga JS hasta el primer evento, y luego solo una vez", async () => {
        const el = makeEl("3");
        const activateFn = vi.fn();
        const loadModule = vi.fn<ModuleLoader>().mockResolvedValue({ default: activateFn });

        const manifest: ActivationManifest = {
            "3": { event: "click", handler: "buy", module: "/assets/P-3.js", strategy: "interaction" },
        };

        initActivation(manifest, { loadModule });
        expect(loadModule).not.toHaveBeenCalled();

        el.dispatchEvent(new Event("click", { bubbles: true }));
        await vi.waitFor(() => expect(loadModule).toHaveBeenCalledTimes(1));

        expect(loadModule).toHaveBeenCalledWith("/assets/P-3.js");
        expect(activateFn).toHaveBeenCalledWith(el);

        // Un segundo click no debe volver a cargar el módulo.
        el.dispatchEvent(new Event("click", { bubbles: true }));
        await Promise.resolve();
        expect(loadModule).toHaveBeenCalledTimes(1);
    });

    it("estrategia 'load': carga inmediatamente, sin esperar ningún evento", async () => {
        const el = makeEl("9");
        const loadModule = vi.fn<ModuleLoader>().mockResolvedValue({ default: vi.fn() });

        const manifest: ActivationManifest = {
            "9": { event: "click", handler: "track", module: "/assets/P-9.js", strategy: "load" },
        };

        initActivation(manifest, { loadModule });
        await vi.waitFor(() => expect(loadModule).toHaveBeenCalledTimes(1));
    });

    it("un id del manifiesto sin elemento correspondiente en el DOM no falla", () => {
        const loadModule = vi.fn<ModuleLoader>();
        const manifest: ActivationManifest = {
            "404": { event: "click", handler: "x", module: "/assets/x.js", strategy: "interaction" },
        };

        expect(() => initActivation(manifest, { loadModule })).not.toThrow();
        expect(loadModule).not.toHaveBeenCalled();
    });

    it("varios elementos con el mismo id (Fase 30, <For>) se activan por separado, cada uno con su propio el", async () => {
        // Un `<For>` clasifica su cuerpo una sola vez — todas las copias
        // que produce el renderer comparten el mismo `data-nexa="<id>"`.
        const first = makeEl("7");
        const second = makeEl("7");
        const third = makeEl("7");

        const activateFn = vi.fn();
        const loadModule = vi.fn<ModuleLoader>().mockResolvedValue({ default: activateFn });

        const manifest: ActivationManifest = {
            "7": { event: "click", handler: "remove", module: "/assets/Catalog-7.js", strategy: "interaction" },
        };

        initActivation(manifest, { loadModule });
        expect(loadModule).not.toHaveBeenCalled();

        // Clickear el del medio solo debe activar (y pasar) ESE elemento —
        // no el primero de la lista, no los tres.
        second.dispatchEvent(new Event("click", { bubbles: true }));
        await vi.waitFor(() => expect(activateFn).toHaveBeenCalledTimes(1));
        expect(activateFn).toHaveBeenCalledWith(second);

        // Los otros dos siguen sin activar hasta que reciban su propio evento.
        first.dispatchEvent(new Event("click", { bubbles: true }));
        third.dispatchEvent(new Event("click", { bubbles: true }));
        await vi.waitFor(() => expect(activateFn).toHaveBeenCalledTimes(3));
        expect(activateFn).toHaveBeenCalledWith(first);
        expect(activateFn).toHaveBeenCalledWith(third);
    });

    it("dispose() cancela las activaciones pendientes", async () => {
        const el = makeEl("1");
        const loadModule = vi.fn<ModuleLoader>().mockResolvedValue({ default: vi.fn() });

        const manifest: ActivationManifest = {
            "1": { event: "click", handler: "buy", module: "/assets/P-1.js", strategy: "interaction" },
        };

        const dispose = initActivation(manifest, { loadModule });
        dispose();

        el.dispatchEvent(new Event("click", { bubbles: true }));
        await Promise.resolve();
        expect(loadModule).not.toHaveBeenCalled();
    });
});
