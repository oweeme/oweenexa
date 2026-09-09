import { describe, expect, it } from "vitest";
import { selectTab } from "../src";

function setupTabs(): { tabUno: HTMLButtonElement; tabDos: HTMLButtonElement; panelUno: HTMLDivElement; panelDos: HTMLDivElement } {
    document.body.innerHTML = `
        <div class="nx-tab-group">
            <div class="nx-tab-list" role="tablist">
                <button class="nx-tab" data-tab="uno" aria-selected="true">Uno</button>
                <button class="nx-tab" data-tab="dos" aria-selected="false">Dos</button>
            </div>
            <div class="nx-tab-panel" data-tab="uno">Contenido uno</div>
            <div class="nx-tab-panel" data-tab="dos" hidden>Contenido dos</div>
        </div>
    `;

    return {
        tabUno: document.querySelector('[data-tab="uno"].nx-tab') as HTMLButtonElement,
        tabDos: document.querySelector('[data-tab="dos"].nx-tab') as HTMLButtonElement,
        panelUno: document.querySelector('[data-tab="uno"].nx-tab-panel') as HTMLDivElement,
        panelDos: document.querySelector('[data-tab="dos"].nx-tab-panel') as HTMLDivElement,
    };
}

function click(el: HTMLElement): void {
    const event = new MouseEvent("click", { bubbles: true });
    Object.defineProperty(event, "currentTarget", { value: el });
    selectTab(event);
}

describe("selectTab", () => {
    it("activa la pestaña clickeada y muestra su panel, ocultando el resto", () => {
        const { tabUno, tabDos, panelUno, panelDos } = setupTabs();
        click(tabDos);

        expect(tabDos.getAttribute("aria-selected")).toBe("true");
        expect(tabUno.getAttribute("aria-selected")).toBe("false");
        expect(panelDos.hidden).toBe(false);
        expect(panelUno.hidden).toBe(true);
    });

    it("solo la pestaña seleccionada queda con tabIndex 0", () => {
        const { tabUno, tabDos } = setupTabs();
        click(tabDos);

        expect(tabDos.tabIndex).toBe(0);
        expect(tabUno.tabIndex).toBe(-1);
    });

    it("no rompe si el evento no viene de un .nx-tab dentro de un .nx-tab-group", () => {
        document.body.innerHTML = `<button>suelto</button>`;
        const loose = document.querySelector("button") as HTMLButtonElement;
        expect(() => click(loose)).not.toThrow();
    });
});
