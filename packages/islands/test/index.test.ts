import { describe, expect, it, vi } from "vitest";
import { initIslands, type IslandModuleLoader, type MountFn } from "../src";

function makeIslandEl(specifier: string, props?: unknown, strategy?: string): HTMLElement {
    const el = document.createElement("div");
    el.setAttribute("data-nexa-island", specifier);
    if (props !== undefined) el.setAttribute("data-nexa-props", JSON.stringify(props));
    if (strategy) el.setAttribute("data-nexa-strategy", strategy);
    document.body.appendChild(el);
    return el;
}

describe("initIslands — Fase 16, criterio de salida", () => {
    it("un documento sin ninguna isla nunca llama a loadModule", () => {
        document.body.innerHTML = "<p>hola</p>";
        const loadModule = vi.fn<IslandModuleLoader>();

        initIslands({ loadModule });

        expect(loadModule).not.toHaveBeenCalled();
    });

    it("sin data-nexa-strategy explícito, se comporta como 'visible' (no como 'interaction')", async () => {
        document.body.innerHTML = "";
        const el = makeIslandEl("productFilter", { products: ["a"] });
        const mount = vi.fn<MountFn>();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: mount });

        // happy-dom no dispara IntersectionObserver de verdad; lo que
        // importa aquí es que NO se dispara por interacción — se
        // verifica que un pointerdown no hace nada, ya que la estrategia
        // activa es 'visible', no 'interaction'.
        initIslands({ loadModule });
        el.dispatchEvent(new Event("pointerdown", { bubbles: true }));
        await Promise.resolve();

        expect(loadModule).not.toHaveBeenCalled();
    });

    it("estrategia 'load': monta de inmediato, con las props ya parseadas del atributo", async () => {
        document.body.innerHTML = "";
        const el = makeIslandEl("dashboardIsland", { title: "Panel" }, "load");
        const mount = vi.fn<MountFn>();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: mount });

        initIslands({ loadModule });

        await vi.waitFor(() => expect(loadModule).toHaveBeenCalledTimes(1));
        expect(loadModule).toHaveBeenCalledWith("dashboardIsland");
        expect(mount).toHaveBeenCalledWith(el, { title: "Panel" });
    });

    it("estrategia 'interaction': espera un evento real sobre el elemento antes de montar", async () => {
        document.body.innerHTML = "";
        const el = makeIslandEl("productFilter", undefined, "interaction");
        const mount = vi.fn<MountFn>();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: mount });

        initIslands({ loadModule });
        expect(loadModule).not.toHaveBeenCalled();

        el.dispatchEvent(new Event("pointerdown", { bubbles: true }));
        await vi.waitFor(() => expect(loadModule).toHaveBeenCalledTimes(1));
    });

    it("sin data-nexa-props, mount recibe un objeto vacío", async () => {
        document.body.innerHTML = "";
        const el = makeIslandEl("dashboardIsland", undefined, "load");
        const mount = vi.fn<MountFn>();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: mount });

        initIslands({ loadModule });

        await vi.waitFor(() => expect(mount).toHaveBeenCalledWith(el, {}));
    });

    it("el cleanup que devuelve mount() se ejecuta al llamar al dispose global", async () => {
        document.body.innerHTML = "";
        makeIslandEl("dashboardIsland", undefined, "load");
        const unmount = vi.fn();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: () => unmount });

        const dispose = initIslands({ loadModule });
        await vi.waitFor(() => expect(loadModule).toHaveBeenCalledTimes(1));

        dispose();
        expect(unmount).toHaveBeenCalledTimes(1);
    });

    it("un data-nexa-props inválido no rompe el montaje — cae a un objeto vacío", async () => {
        document.body.innerHTML = "";
        const el = document.createElement("div");
        el.setAttribute("data-nexa-island", "dashboardIsland");
        el.setAttribute("data-nexa-props", "{not valid json");
        el.setAttribute("data-nexa-strategy", "load");
        document.body.appendChild(el);

        const mount = vi.fn<MountFn>();
        const loadModule = vi.fn<IslandModuleLoader>().mockResolvedValue({ default: mount });

        initIslands({ loadModule });

        await vi.waitFor(() => expect(mount).toHaveBeenCalledWith(el, {}));
    });
});
