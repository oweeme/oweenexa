import { describe, expect, it } from "vitest";
import { createPageSearch } from "../src";

function setupCatalog(): HTMLDivElement {
    document.body.innerHTML = `
        <div id="catalogo">
            <article data-searchable>Zapatillas Nezha</article>
            <article data-searchable>Camiseta Jade</article>
            <article data-searchable>Gorra roja</article>
        </div>
    `;
    return document.getElementById("catalogo") as HTMLDivElement;
}

function items(container: HTMLElement): HTMLElement[] {
    return Array.from(container.querySelectorAll<HTMLElement>("[data-searchable]"));
}

describe("createPageSearch", () => {
    it("con query vacía, no oculta nada", () => {
        const container = setupCatalog();
        createPageSearch(container);

        expect(items(container).every((item) => !item.hidden)).toBe(true);
    });

    it("setQuery oculta reactivamente los items que no matchean", async () => {
        const container = setupCatalog();
        const search = createPageSearch(container);

        search.setQuery("jade");
        await Promise.resolve();

        expect(items(container).map((item) => item.hidden)).toEqual([true, false, true]);
    });

    it("no distingue mayúsculas/minúsculas", async () => {
        const container = setupCatalog();
        const search = createPageSearch(container);

        search.setQuery("ZAPATILLAS");
        await Promise.resolve();

        expect(items(container).map((item) => item.hidden)).toEqual([false, true, true]);
    });

    it("volver a vaciar la query muestra todo de nuevo (reactividad real, no una sola pasada)", async () => {
        const container = setupCatalog();
        const search = createPageSearch(container);

        search.setQuery("jade");
        await Promise.resolve();
        search.setQuery("");
        await Promise.resolve();

        expect(items(container).every((item) => !item.hidden)).toBe(true);
    });

    it("acepta un selector de item distinto al default", async () => {
        document.body.innerHTML = `
            <ul id="lista">
                <li class="producto">Mate</li>
                <li class="producto">Termo</li>
            </ul>
        `;
        const container = document.getElementById("lista") as HTMLUListElement;
        const search = createPageSearch(container, ".producto");

        search.setQuery("termo");
        await Promise.resolve();

        const lis = Array.from(container.querySelectorAll<HTMLElement>(".producto"));
        expect(lis.map((li) => li.hidden)).toEqual([true, false]);
    });
});
