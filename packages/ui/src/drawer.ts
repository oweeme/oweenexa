/**
 * `ui.openDrawer(el)` / `ui.closeDrawer(el)` (Fase 41, issue #9) — mismo
 * contrato de accesibilidad que `Dialog` (foco atrapado, Escape,
 * devolver el foco), reutilizado vía `focus-trap.ts`; lo único distinto
 * es el layout visual (`drawer.css`: panel lateral deslizante en vez de
 * una caja centrada).
 *
 * Navegación SPA (`initRouter`, Fase 22/32): el estado vive en dos
 * `WeakMap` clavadas por el propio nodo del drawer, no en variables
 * globales. Si el drawer está dentro del `[data-nexa-slot]` que
 * `initRouter` reemplaza, el nodo viejo (y sus entradas en los WeakMap)
 * simplemente se descartan con la navegación — sin fugas, sin listener
 * huérfano. Si el drawer vive en el layout persistente (fuera del slot,
 * caso típico de un menú lateral), sobrevive intacto porque
 * `initRouter` no lo toca; queda abierto entre páginas, que es el
 * comportamiento esperado de una navegación persistente. En ambos casos
 * `closeDrawer` sigue siendo seguro de llamar dos veces o sobre un nodo
 * ya desconectado del DOM (`toRestore?.focus()` sobre un nodo separado
 * del documento no lanza).
 */

import { getFocusableElements, trapFocus } from "./focus-trap";

const previouslyFocused = new WeakMap<HTMLElement, HTMLElement | null>();
const keydownListeners = new WeakMap<HTMLElement, (event: KeyboardEvent) => void>();

export function openDrawer(drawer: HTMLElement): void {
    previouslyFocused.set(drawer, document.activeElement as HTMLElement | null);

    drawer.removeAttribute("hidden");
    drawer.setAttribute("role", "dialog");
    drawer.setAttribute("aria-modal", "true");

    const focusable = getFocusableElements(drawer);
    (focusable[0] ?? drawer).focus();

    const onKeydown = (event: KeyboardEvent) => handleKeydown(drawer, event);
    keydownListeners.set(drawer, onKeydown);
    drawer.addEventListener("keydown", onKeydown);
}

export function closeDrawer(drawer: HTMLElement): void {
    drawer.setAttribute("hidden", "");

    const listener = keydownListeners.get(drawer);
    if (listener) {
        drawer.removeEventListener("keydown", listener);
        keydownListeners.delete(drawer);
    }

    const toRestore = previouslyFocused.get(drawer);
    previouslyFocused.delete(drawer);
    toRestore?.focus();
}

function handleKeydown(drawer: HTMLElement, event: KeyboardEvent): void {
    if (event.key === "Escape") {
        closeDrawer(drawer);
        return;
    }
    if (event.key === "Tab") {
        trapFocus(drawer, event);
    }
}
