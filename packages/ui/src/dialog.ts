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
 */

const previouslyFocused = new WeakMap<HTMLElement, HTMLElement | null>();
const keydownListeners = new WeakMap<HTMLElement, (event: KeyboardEvent) => void>();

const FOCUSABLE_SELECTOR =
    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function openDialog(dialog: HTMLElement): void {
    previouslyFocused.set(dialog, document.activeElement as HTMLElement | null);

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
    dialog.setAttribute("hidden", "");

    const listener = keydownListeners.get(dialog);
    if (listener) {
        dialog.removeEventListener("keydown", listener);
        keydownListeners.delete(dialog);
    }

    const toRestore = previouslyFocused.get(dialog);
    previouslyFocused.delete(dialog);
    toRestore?.focus();
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

function trapFocus(dialog: HTMLElement, event: KeyboardEvent): void {
    const focusable = getFocusableElements(dialog);
    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
    }
}

function getFocusableElements(container: HTMLElement): HTMLElement[] {
    return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
}
