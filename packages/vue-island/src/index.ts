import { createApp, type Component } from "vue";

/**
 * Referencia de "paquete de comunidad" (Fase 16), en el mismo espíritu
 * que `@nexa/stripe` (Fase 15): un adaptador real y delgado sobre Vue 3
 * de verdad (`vue`, npm), no escrito ni mantenido por el core de Nexa —
 * exactamente lo que `@nexa/vue-island`, `@nexa/preact-island`, etc.
 * serían en la práctica.
 *
 * `defineVueIsland` traduce un componente Vue cualquiera al contrato de
 * montaje que espera `@nexa/islands` (`mount(el, props)` →
 * `void | (() => void)`). Nexa nunca ve el componente Vue en sí — solo
 * ve el módulo resultante como un `data-nexa-island="..."` opaco.
 */
export function defineVueIsland(component: Component) {
    return function mount(el: Element, props: Record<string, unknown>): () => void {
        const app = createApp(component, props);
        app.mount(el);
        return () => app.unmount();
    };
}
