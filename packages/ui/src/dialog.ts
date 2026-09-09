/**
 * Comportamiento accesible de `.nx-dialog`: no es opcional, es parte del
 * componente (ver docs/FASES-DE-CONSTRUCCION.md, Fase 9). Maneja:
 * - `role="dialog"` / `aria-modal="true"` al abrir.
 * - foco inicial dentro del diálogo, y devolución del foco al cerrar.
 * - atrapar el `Tab` dentro del diálogo (focus trap).
 * - `Escape` para cerrar.
 *
 * El HTML lo escribe el desarrollador (`<div class="nx-dialog" hidden>`);
 * esto solo activa el comportamiento sobre ese elemento — el mismo
 * principio de Progressive Activation que usa el resto del framework.
 *
 * Apilamiento (Fase 42, issue #10): cada diálogo es un elemento propio,
 * así que el focus trap ya estaba aislado por diálogo desde la Fase 9
 * (`trapFocus(dialog, ...)` solo mira dentro de `dialog`, y un `keydown`
 * dentro de B nunca burbujea a través de A porque no son ancestro/
 * descendiente). Lo que faltaba era el z-index (para que B se vea
 * encima de A sin depender del orden de inserción en el DOM) y ocultar
 * A de lectores de pantalla mientras B lo cubre (`aria-hidden`) — eso
 * es lo que agrega `dialogStack` acá.
 */

import { getFocusableElements, trapFocus } from "./focus-trap";

const previouslyFocused = new WeakMap<HTMLElement, HTMLElement | null>();
const keydownListeners = new WeakMap<HTMLElement, (event: KeyboardEvent) => void>();
const closeCallbacks = new WeakMap<HTMLElement, () => void>();
const dialogStack: HTMLElement[] = [];

const BASE_Z_INDEX = 1000;
const Z_INDEX_STEP = 10;

export function openDialog(dialog: HTMLElement): void {
    previouslyFocused.set(dialog, document.activeElement as HTMLElement | null);

    const below = dialogStack[dialogStack.length - 1];
    below?.setAttribute("aria-hidden", "true");

    dialog.style.zIndex = String(BASE_Z_INDEX + dialogStack.length * Z_INDEX_STEP);
    dialogStack.push(dialog);

    dialog.removeAttribute("hidden");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");

    const focusable = getFocusableElements(dialog);
    (focusable[0] ?? dialog).focus();

    const onKeydown = (event: KeyboardEvent) => handleKeydown(dialog, event);
    keydownListeners.set(dialog, onKeydown);
    dialog.addEventListener("keydown", onKeydown);
}

export function closeDialog(dialog: HTMLElement): void {
    const stackIndex = dialogStack.indexOf(dialog);
    if (stackIndex !== -1) {
        dialogStack.splice(stackIndex, 1);
    }
    dialog.style.removeProperty("z-index");
    dialogStack[dialogStack.length - 1]?.removeAttribute("aria-hidden");

    dialog.setAttribute("hidden", "");

    const listener = keydownListeners.get(dialog);
    if (listener) {
        dialog.removeEventListener("keydown", listener);
        keydownListeners.delete(dialog);
    }

    const onClose = closeCallbacks.get(dialog);
    closeCallbacks.delete(dialog);

    const toRestore = previouslyFocused.get(dialog);
    previouslyFocused.delete(dialog);
    toRestore?.focus();

    onClose?.();
}

/**
 * Punto de extensión interno, usado por `confirm()`/`alert()`
 * (`confirm-dialog.ts`) para enterarse de que un diálogo se cerró sin
 * importar la vía (botón o Escape) y así resolver su promesa. No forma
 * parte de la API pública de `@nexa/ui` — no se reexporta desde
 * `index.ts`.
 */
export function onDialogClose(dialog: HTMLElement, callback: () => void): void {
    closeCallbacks.set(dialog, callback);
}

function handleKeydown(dialog: HTMLElement, event: KeyboardEvent): void {
    if (event.key === "Escape") {
        closeDialog(dialog);
        return;
    }
    if (event.key === "Tab") {
        trapFocus(dialog, event);
    }
}
