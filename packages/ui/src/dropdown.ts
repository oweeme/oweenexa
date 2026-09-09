/**
 * Dropdown/select simple, sin isla: abre/cierra un menú de opciones,
 * se cierra con click afuera o Escape, y deja la opción elegida como
 * texto del trigger. Mismo principio que Dialog (Fase 9) — el foco no
 * se atrapa como en un modal a propósito, porque un dropdown no bloquea
 * el resto de la página.
 *
 * Marcado esperado:
 * ```
 * <div class="nx-dropdown">
 *   <button class="nx-dropdown-trigger" aria-haspopup="listbox" aria-expanded="false" onClick={toggleDropdown}>
 *     Elegir…
 *   </button>
 *   <ul class="nx-dropdown-menu" role="listbox" hidden>
 *     <li class="nx-dropdown-option" role="option" data-value="a" onClick={selectDropdownOption}>Opción A</li>
 *     <li class="nx-dropdown-option" role="option" data-value="b" onClick={selectDropdownOption}>Opción B</li>
 *   </ul>
 * </div>
 * ```
 */
const outsideClickListeners = new WeakMap<HTMLElement, (event: MouseEvent) => void>();
const keydownListeners = new WeakMap<HTMLElement, (event: KeyboardEvent) => void>();

export function toggleDropdown(event: Event): void {
    const trigger = event.currentTarget as HTMLElement | null;
    const dropdown = trigger?.closest<HTMLElement>(".nx-dropdown");
    const menu = dropdown?.querySelector<HTMLElement>(".nx-dropdown-menu");
    if (!trigger || !dropdown || !menu) return;

    if (menu.hasAttribute("hidden")) {
        openDropdownMenu(dropdown, trigger, menu);
    } else {
        closeDropdownMenu(dropdown, trigger, menu);
    }
}

export function selectDropdownOption(event: Event): void {
    const option = event.currentTarget as HTMLElement | null;
    const dropdown = option?.closest<HTMLElement>(".nx-dropdown");
    const trigger = dropdown?.querySelector<HTMLElement>(".nx-dropdown-trigger");
    const menu = dropdown?.querySelector<HTMLElement>(".nx-dropdown-menu");
    if (!option || !dropdown || !trigger || !menu) return;

    for (const candidate of menu.querySelectorAll<HTMLElement>(".nx-dropdown-option")) {
        candidate.setAttribute("aria-selected", String(candidate === option));
    }
    trigger.textContent = option.textContent;
    if (option.dataset.value !== undefined) {
        trigger.dataset.value = option.dataset.value;
    }

    closeDropdownMenu(dropdown, trigger, menu);
}

function openDropdownMenu(dropdown: HTMLElement, trigger: HTMLElement, menu: HTMLElement): void {
    menu.removeAttribute("hidden");
    trigger.setAttribute("aria-expanded", "true");

    const onOutsideClick = (clickEvent: MouseEvent) => {
        if (!dropdown.contains(clickEvent.target as Node)) {
            closeDropdownMenu(dropdown, trigger, menu);
        }
    };
    const onKeydown = (keyEvent: KeyboardEvent) => {
        if (keyEvent.key === "Escape") {
            closeDropdownMenu(dropdown, trigger, menu);
            trigger.focus();
        }
    };

    outsideClickListeners.set(dropdown, onOutsideClick);
    keydownListeners.set(dropdown, onKeydown);
    document.addEventListener("click", onOutsideClick);
    document.addEventListener("keydown", onKeydown);
}

function closeDropdownMenu(dropdown: HTMLElement, trigger: HTMLElement, menu: HTMLElement): void {
    menu.setAttribute("hidden", "");
    trigger.setAttribute("aria-expanded", "false");

    const onOutsideClick = outsideClickListeners.get(dropdown);
    if (onOutsideClick) {
        document.removeEventListener("click", onOutsideClick);
        outsideClickListeners.delete(dropdown);
    }
    const onKeydown = keydownListeners.get(dropdown);
    if (onKeydown) {
        document.removeEventListener("keydown", onKeydown);
        keydownListeners.delete(dropdown);
    }
}
