import { defineVueIsland } from "@nexa/vue-island";
import { Dashboard } from "./Dashboard";

/**
 * `@nexa/vue-island` (Fase 16) traduce `Dashboard` (un componente Vue 3
 * real) al contrato de montaje `mount(el, props)` que espera
 * `@nexa/islands`. Nexa nunca ve `Dashboard.ts` — solo ve este módulo
 * como un `data-nexa-island="dashboardIsland"` opaco.
 */
export default defineVueIsland(Dashboard);
