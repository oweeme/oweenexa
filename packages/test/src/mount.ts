/**
 * Lo que exporta por defecto un chunk generado por Nexa (ver
 * `crates/nexa-activation/src/chunk.rs::real_chunk`): recibe el elemento
 * ya presente en el DOM y le engancha su handler.
 */
export type ChunkActivate = (el: Element) => void;

export interface MountedChunk {
    /** El elemento raíz del fragmento montado — ya activado. */
    el: HTMLElement;
    /** Quita `el` del documento. Llamarlo al final de cada test evita
     * que un test deje basura en el DOM que afecte al siguiente. */
    destroy(): void;
}

export interface MountChunkOptions {
    document?: Document;
}

/**
 * Monta un fragmento de HTML (el que produciría el renderer de Nexa
 * para un nodo interactivo) en un contenedor real del DOM y le aplica el
 * `activate` de su chunk — exactamente lo que hace
 * `@nexa/runtime::initActivation` en el navegador, pero sin depender de
 * un manifiesto ni de una estrategia: aquí se activa siempre, de
 * inmediato, porque el punto es probar el handler en aislamiento, no el
 * mecanismo de activación en sí (eso ya lo prueba `@nexa/runtime`).
 *
 * Lanza si el HTML no tiene un único elemento raíz — un chunk siempre se
 * engancha a un elemento concreto (`el.addEventListener(...)`), montar
 * texto suelto o varios hermanos no tiene un "el" que activar.
 */
export function mountChunk(activate: ChunkActivate, html: string, options: MountChunkOptions = {}): MountedChunk {
    const doc = options.document ?? document;
    const container = doc.createElement("div");
    container.innerHTML = html.trim();

    if (container.children.length !== 1) {
        throw new Error(
            `[nexa/test] mountChunk espera un único elemento raíz en el HTML; encontró ${container.children.length}.`,
        );
    }

    const el = container.firstElementChild as HTMLElement;
    doc.body.appendChild(container);
    activate(el);

    return {
        el,
        destroy: () => container.remove(),
    };
}
