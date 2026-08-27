import { describe, expect, it } from "vitest";
import { closeDialog, openDialog } from "../src";

function setupDialog(): { trigger: HTMLButtonElement; dialog: HTMLDivElement; first: HTMLButtonElement; last: HTMLButtonElement } {
    document.body.innerHTML = "";

    const trigger = document.createElement("button");
    trigger.textContent = "Abrir";
    document.body.appendChild(trigger);

    const dialog = document.createElement("div");
    dialog.className = "nx-dialog";
    dialog.setAttribute("hidden", "");
    document.body.appendChild(dialog);

    const first = document.createElement("button");
    first.textContent = "Primero";
    const last = document.createElement("button");
    last.textContent = "Último";
    dialog.append(first, last);

    trigger.focus();

    return { trigger, dialog, first, last };
}

describe("openDialog / closeDialog", () => {
    it("quita `hidden` y pone los atributos ARIA de un diálogo modal", () => {
        const { dialog } = setupDialog();
        openDialog(dialog);

        expect(dialog.hasAttribute("hidden")).toBe(false);
        expect(dialog.getAttribute("role")).toBe("dialog");
        expect(dialog.getAttribute("aria-modal")).toBe("true");
    });

    it("mueve el foco dentro del diálogo al abrir", () => {
        const { dialog, first } = setupDialog();
        openDialog(dialog);

        expect(document.activeElement).toBe(first);
    });

    it("devuelve el foco a quien abrió el diálogo, al cerrarlo", () => {
        const { trigger, dialog } = setupDialog();
        openDialog(dialog);
        closeDialog(dialog);

        expect(dialog.hasAttribute("hidden")).toBe(true);
        expect(document.activeElement).toBe(trigger);
    });

    it("Escape cierra el diálogo", () => {
        const { dialog } = setupDialog();
        openDialog(dialog);

        dialog.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

        expect(dialog.hasAttribute("hidden")).toBe(true);
    });

    it("Tab en el último elemento enfocable vuelve al primero (focus trap)", () => {
        const { dialog, first, last } = setupDialog();
        openDialog(dialog);
        last.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
        dialog.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(first);
    });

    it("Shift+Tab en el primer elemento enfocable va al último (focus trap)", () => {
        const { dialog, first, last } = setupDialog();
        openDialog(dialog);
        first.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true });
        dialog.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(last);
    });
});
