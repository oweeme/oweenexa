/**
 * Comportamiento de un grupo de pestañas (`.nx-tab-group`): activa la
 * pestaña clickeada y muestra su panel — nunca una isla, es una función
 * normal llamada desde un `onClick` (Fase 57, docs/FASES-DE-CONSTRUCCION.md),
 * igual que Dialog/Drawer desde la Fase 9.
 *
 * Marcado esperado:
 * ```
 * <div class="nx-tab-group">
 *   <div class="nx-tab-list" role="tablist">
 *     <button class="nx-tab" data-tab="uno" aria-selected="true" onClick={selectTab}>Uno</button>
 *     <button class="nx-tab" data-tab="dos" onClick={selectTab}>Dos</button>
 *   </div>
 *   <div class="nx-tab-panel" data-tab="uno">Contenido uno</div>
 *   <div class="nx-tab-panel" data-tab="dos" hidden>Contenido dos</div>
 * </div>
 * ```
 * (donde `function selectTab(event) { ui.selectTab(event); }` en la página)
 */
export function selectTab(event: Event): void {
    const tab = event.currentTarget as HTMLElement | null;
    const group = tab?.closest<HTMLElement>(".nx-tab-group");
    const tabId = tab?.dataset.tab;
    if (!group || tabId === undefined) return;

    for (const candidate of group.querySelectorAll<HTMLElement>(".nx-tab")) {
        const selected = candidate.dataset.tab === tabId;
        candidate.setAttribute("aria-selected", String(selected));
        candidate.tabIndex = selected ? 0 : -1;
    }

    for (const panel of group.querySelectorAll<HTMLElement>(".nx-tab-panel")) {
        panel.hidden = panel.dataset.tab !== tabId;
    }
}
