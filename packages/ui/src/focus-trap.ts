/**
 * Piezas de foco compartidas entre `Dialog` y `Drawer` (Fase 41,
 * issue #9) — un drawer es, en términos de accesibilidad, el mismo
 * contrato que un diálogo modal (foco atrapado, Escape, devolver el
 * foco), solo cambia el layout visual. Extraído de `dialog.ts` para no
 * duplicar la lógica de focus trap entre los dos componentes.
 */

export const FOCUSABLE_SELECTOR =
    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function getFocusableElements(container: HTMLElement): HTMLElement[] {
    return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
}

export function trapFocus(container: HTMLElement, event: KeyboardEvent): void {
    const focusable = getFocusableElements(container);
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
