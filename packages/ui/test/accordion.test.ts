import { describe, expect, it } from "vitest";
import { toggleAccordionItem } from "../src";

function click(el: HTMLElement): void {
    const event = new MouseEvent("click", { bubbles: true });
    Object.defineProperty(event, "currentTarget", { value: el });
    toggleAccordionItem(event);
}

describe("toggleAccordionItem", () => {
    it("expande un panel colapsado y lo marca aria-expanded", () => {
        document.body.innerHTML = `
            <div class="nx-accordion">
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="false">Pregunta</button>
                    <div class="nx-accordion-panel" hidden>Respuesta</div>
                </div>
            </div>
        `;
        const trigger = document.querySelector(".nx-accordion-trigger") as HTMLButtonElement;
        const panel = document.querySelector(".nx-accordion-panel") as HTMLDivElement;

        click(trigger);

        expect(trigger.getAttribute("aria-expanded")).toBe("true");
        expect(panel.hidden).toBe(false);
    });

    it("colapsa un panel ya expandido", () => {
        document.body.innerHTML = `
            <div class="nx-accordion">
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="true">Pregunta</button>
                    <div class="nx-accordion-panel">Respuesta</div>
                </div>
            </div>
        `;
        const trigger = document.querySelector(".nx-accordion-trigger") as HTMLButtonElement;
        const panel = document.querySelector(".nx-accordion-panel") as HTMLDivElement;

        click(trigger);

        expect(trigger.getAttribute("aria-expanded")).toBe("false");
        expect(panel.hidden).toBe(true);
    });

    it("por defecto, varios items pueden estar abiertos a la vez", () => {
        document.body.innerHTML = `
            <div class="nx-accordion">
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="true">Uno</button>
                    <div class="nx-accordion-panel">Respuesta uno</div>
                </div>
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="false">Dos</button>
                    <div class="nx-accordion-panel" hidden>Respuesta dos</div>
                </div>
            </div>
        `;
        const [triggerUno, triggerDos] = Array.from(document.querySelectorAll(".nx-accordion-trigger")) as HTMLButtonElement[];

        click(triggerDos);

        expect(triggerUno.getAttribute("aria-expanded")).toBe("true");
        expect(triggerDos.getAttribute("aria-expanded")).toBe("true");
    });

    it("con data-exclusive=true, abrir un item cierra los demás", () => {
        document.body.innerHTML = `
            <div class="nx-accordion" data-exclusive="true">
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="true">Uno</button>
                    <div class="nx-accordion-panel">Respuesta uno</div>
                </div>
                <div class="nx-accordion-item">
                    <button class="nx-accordion-trigger" aria-expanded="false">Dos</button>
                    <div class="nx-accordion-panel" hidden>Respuesta dos</div>
                </div>
            </div>
        `;
        const [triggerUno, triggerDos] = Array.from(document.querySelectorAll(".nx-accordion-trigger")) as HTMLButtonElement[];
        const [panelUno, panelDos] = Array.from(document.querySelectorAll(".nx-accordion-panel")) as HTMLDivElement[];

        click(triggerDos);

        expect(triggerDos.getAttribute("aria-expanded")).toBe("true");
        expect(panelDos.hidden).toBe(false);
        expect(triggerUno.getAttribute("aria-expanded")).toBe("false");
        expect(panelUno.hidden).toBe(true);
    });
});
