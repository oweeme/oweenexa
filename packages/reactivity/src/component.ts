/**
 * Modelo de componentes liviano (Fase 50, issue #19) — **solo para
 * código dentro de una isla escrita en formato nativo con
 * `@nexa/reactivity`** (Fase 5/16). Una página Nexa Core sigue siendo
 * "un componente por página, sin `<Otro/>`, sin props" — esa regla no
 * cambia acá, y este módulo no tiene ninguna forma de usarse desde
 * `.tsx` de Nexa (no lo entiende `nexa-parser`, no genera nada en el
 * servidor). Esto es exclusivamente una utilidad de runtime para
 * organizar el código *dentro* del `mount(el, props)` que ya espera
 * `@nexa/islands` (Fase 16).
 *
 * Sigue siendo, letra por letra, "señales que notifican directo, sin
 * Virtual DOM" — un `Component<P>` es una función que manipula DOM
 * real una sola vez al montarse (nunca se "re-renderiza" a sí mismo).
 * Lo que gana con esto es exclusivamente **composición y limpieza
 * encadenada**: un componente puede montar otro dentro de un
 * contenedor propio, y desmontar el padre desmonta automáticamente
 * todo lo que montó (hijos, `effect()`s, listeners registrados con
 * `onCleanup`) — sin tener que devolver y combinar funciones de
 * limpieza a mano en cada nivel, que es exactamente el código
 * repetitivo que ya se ve en `productFilter.island.ts`.
 *
 * Reactividad entre componentes, sin ningún mecanismo nuevo: una
 * `Signal` es un valor como cualquier otro, así que pasarla como prop
 * y leerla con `.value` dentro de un `effect()` del hijo ya alcanza
 * para que el hijo reaccione a cambios del padre — no hace falta un
 * sistema de props reactivas aparte.
 */

export interface ComponentContext {
    /** Registra una función de limpieza para cuando este componente se desmonte. */
    onCleanup(fn: () => void): void;
    /**
     * Monta `component` dentro de `container` con `props` — su
     * limpieza queda encadenada a la de este componente: cuando el
     * padre se desmonta, el hijo se desmonta también, en el orden
     * inverso al que se montó.
     */
    mount<P>(component: Component<P>, container: Element, props: P): void;
}

/**
 * Un componente: una función que recibe el elemento donde vive, sus
 * props (tipadas, un objeto simple — nunca una `Signal` implícita,
 * aunque nada impide pasar una `Signal` como valor de una prop), y un
 * `ComponentContext` para registrar limpieza y montar hijos.
 */
export type Component<P> = (container: Element, props: P, ctx: ComponentContext) => void;

/**
 * Monta `component` en `container` — devuelve una función de limpieza
 * que desmonta, en orden inverso, todo lo que se registró con
 * `ctx.onCleanup`/`ctx.mount` durante el montaje (incluidos los
 * componentes hijos, transitivamente).
 */
export function mountComponent<P>(component: Component<P>, container: Element, props: P): () => void {
    const cleanups: Array<() => void> = [];

    const ctx: ComponentContext = {
        onCleanup(fn) {
            cleanups.push(fn);
        },
        mount(child, childContainer, childProps) {
            cleanups.push(mountComponent(child, childContainer, childProps));
        },
    };

    component(container, props, ctx);

    let disposed = false;
    return () => {
        if (disposed) return;
        disposed = true;
        for (const cleanup of cleanups.splice(0).reverse()) {
            cleanup();
        }
    };
}
