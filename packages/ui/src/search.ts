import { effect, state } from "@nexa/reactivity";

/**
 * Buscador reactivo de página completa: filtra cualquier conjunto de
 * elementos marcados con `data-searchable` dentro de `container`, con
 * un `Signal` real de `@nexa/reactivity` (Fase 4) por debajo — nunca
 * una isla. El efecto vive en el mismo chunk que activó el `<input>`,
 * operando sobre el DOM que el servidor ya renderizó (sin re-fetch).
 *
 * Se crea perezosamente dentro del propio handler de `input`, la
 * primera vez que el usuario escribe. Ojo: el chunk que Nexa extrae de
 * `onSearchInput` es un módulo aislado — no puede depender de una
 * variable de módulo declarada aparte en la página (esa `let` no viaja
 * con el chunk). Por eso el controlador se guarda colgado del propio
 * `<input>` (`event.currentTarget`), que sí persiste entre una tecleada
 * y la siguiente:
 * ```
 * function onSearchInput(event) {
 *     const input = event.currentTarget;
 *     input.pageSearch ??= ui.createPageSearch(document.getElementById("catalogo"));
 *     input.pageSearch.setQuery(event.target.value);
 * }
 * <input type="search" onInput={onSearchInput} placeholder="Buscar…" />
 * <div id="catalogo">
 *     <article data-searchable>Zapatillas Nezha</article>
 *     <article data-searchable>Camiseta Jade</article>
 * </div>
 * ```
 */
export interface PageSearch {
    setQuery(value: string): void;
}

export function createPageSearch(container: Element, itemSelector = "[data-searchable]"): PageSearch {
    const query = state("");

    effect(() => {
        const q = query.value.trim().toLowerCase();
        for (const item of Array.from(container.querySelectorAll<HTMLElement>(itemSelector))) {
            const text = (item.textContent ?? "").toLowerCase();
            item.hidden = q.length > 0 && !text.includes(q);
        }
    });

    return {
        setQuery(value: string): void {
            query.value = value;
        },
    };
}
