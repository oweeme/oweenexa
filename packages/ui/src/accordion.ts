/**
 * Expandir/colapsar un item de acordeón (`.nx-accordion-item`) — sin
 * isla, misma idea que `tabs.ts`. Con `data-exclusive="true"` en el
 * `.nx-accordion` contenedor, abrir un item cierra los demás (acordeón
 * de un solo panel abierto); sin ese atributo, varios pueden estar
 * abiertos a la vez (comportamiento por defecto).
 *
 * Marcado esperado:
 * ```
 * <div class="nx-accordion">
 *   <div class="nx-accordion-item">
 *     <button class="nx-accordion-trigger" aria-expanded="false" onClick={toggleAccordionItem}>Pregunta</button>
 *     <div class="nx-accordion-panel" hidden>Respuesta</div>
 *   </div>
 * </div>
 * ```
 */
export function toggleAccordionItem(event: Event): void {
    const trigger = event.currentTarget as HTMLElement | null;
    const item = trigger?.closest<HTMLElement>(".nx-accordion-item");
    const accordion = item?.closest<HTMLElement>(".nx-accordion");
    const panel = item?.querySelector<HTMLElement>(".nx-accordion-panel");
    if (!trigger || !item || !panel) return;

    const expanded = trigger.getAttribute("aria-expanded") === "true";

    if (!expanded && accordion?.dataset.exclusive === "true") {
        for (const otherItem of accordion.querySelectorAll<HTMLElement>(".nx-accordion-item")) {
            if (otherItem === item) continue;
            otherItem.querySelector<HTMLElement>(".nx-accordion-trigger")?.setAttribute("aria-expanded", "false");
            const otherPanel = otherItem.querySelector<HTMLElement>(".nx-accordion-panel");
            if (otherPanel) otherPanel.hidden = true;
        }
    }

    trigger.setAttribute("aria-expanded", String(!expanded));
    panel.hidden = expanded;
}
