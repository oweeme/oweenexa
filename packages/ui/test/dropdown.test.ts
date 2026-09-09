import { describe, expect, it } from "vitest";
import { selectDropdownOption, toggleDropdown } from "../src";

function setup(): { trigger: HTMLButtonElement; menu: HTMLUListElement; optionA: HTMLLIElement; optionB: HTMLLIElement } {
    document.body.innerHTML = `
        <div class="nx-dropdown">
            <button class="nx-dropdown-trigger" aria-haspopup="listbox" aria-expanded="false">Elegir…</button>
            <ul class="nx-dropdown-menu" role="listbox" hidden>
                <li class="nx-dropdown-option" role="option" data-value="a">Opción A</li>
                <li class="nx-dropdown-option" role="option" data-value="b">Opción B</li>
            </ul>
        </div>
    `;

    return {
        trigger: document.querySelector(".nx-dropdown-trigger") as HTMLButtonElement,
        menu: document.querySelector(".nx-dropdown-menu") as HTMLUListElement,
        optionA: document.querySelector('[data-value="a"]') as HTMLLIElement,
        optionB: document.querySelector('[data-value="b"]') as HTMLLIElement,
    };
}

function click(el: HTMLElement, fn: (event: Event) => void): void {
    const event = new MouseEvent("click", { bubbles: true, cancelable: true });
    Object.defineProperty(event, "currentTarget", { value: el });
    fn(event);
}

describe("toggleDropdown / selectDropdownOption", () => {
    it("abre el menú y marca aria-expanded", () => {
        const { trigger, menu } = setup();
        click(trigger, toggleDropdown);

        expect(menu.hasAttribute("hidden")).toBe(false);
        expect(trigger.getAttribute("aria-expanded")).toBe("true");
    });

    it("cierra el menú si se vuelve a clickear el trigger", () => {
        const { trigger, menu } = setup();
        click(trigger, toggleDropdown);
        click(trigger, toggleDropdown);

        expect(menu.hasAttribute("hidden")).toBe(true);
        expect(trigger.getAttribute("aria-expanded")).toBe("false");
    });

    it("Escape cierra el menú y devuelve el foco al trigger", () => {
        const { trigger, menu } = setup();
        click(trigger, toggleDropdown);

        document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

        expect(menu.hasAttribute("hidden")).toBe(true);
        expect(document.activeElement).toBe(trigger);
    });

    it("click afuera cierra el menú", () => {
        const { trigger, menu } = setup();
        click(trigger, toggleDropdown);

        const outside = document.createElement("div");
        document.body.appendChild(outside);
        outside.dispatchEvent(new MouseEvent("click", { bubbles: true }));

        expect(menu.hasAttribute("hidden")).toBe(true);
    });

    it("elegir una opción actualiza el texto del trigger, marca aria-selected y cierra el menú", () => {
        const { trigger, menu, optionA, optionB } = setup();
        click(trigger, toggleDropdown);
        click(optionB, selectDropdownOption);

        expect(trigger.textContent).toBe("Opción B");
        expect(trigger.dataset.value).toBe("b");
        expect(optionB.getAttribute("aria-selected")).toBe("true");
        expect(optionA.getAttribute("aria-selected")).toBe("false");
        expect(menu.hasAttribute("hidden")).toBe(true);
    });
});
